import argparse
import json
import math
from pathlib import Path

import numpy as np
import pandas as pd
import scipy.signal
import sympy
import sympybotics
from sympybotics._compatibility_ import exec_


DOF = 7


def make_franka_robotdef(friction=True):
    dh = [
        (0, 0, 0.333, "q"),
        ("-pi/2", 0, 0, "q"),
        ("pi/2", 0, 0.316, "q"),
        ("pi/2", 0.0825, 0, "q"),
        ("-pi/2", -0.0825, 0.384, "q"),
        ("pi/2", 0, 0, "q"),
        ("pi/2", 0.088, 0, "q"),
    ]
    robotdef = sympybotics.RobotDef("Franka Panda", dh, dh_convention="modified")
    robotdef.gravityacc = sympy.Matrix([0.0, 0.0, -9.81])
    if friction:
        robotdef.frictionmodel = {"viscous", "Coulomb", "offset"}
    return robotdef


def load_log(path):
    data = pd.read_csv(path)
    q = data[[f"q{i}" for i in range(DOF)]].to_numpy(dtype=float)
    dq = data[[f"dq{i}" for i in range(DOF)]].to_numpy(dtype=float)
    ddq = data[[f"ddq_est{i}" for i in range(DOF)]].to_numpy(dtype=float)
    tau = data[[f"tau_j{i}" for i in range(DOF)]].to_numpy(dtype=float)
    time = data["t_s"].to_numpy(dtype=float)
    valid = np.isfinite(q).all(axis=1)
    valid &= np.isfinite(dq).all(axis=1)
    valid &= np.isfinite(ddq).all(axis=1)
    valid &= np.isfinite(tau).all(axis=1)
    return time[valid], q[valid], dq[valid], ddq[valid], tau[valid]


def smooth_kinematics(time, q, window_seconds, polyorder):
    if window_seconds <= 0:
        return None

    dt = float(np.median(np.diff(time)))
    window = max(polyorder + 2, int(round(window_seconds / dt)))
    if window % 2 == 0:
        window += 1
    if window >= len(q):
        window = len(q) - 1 if len(q) % 2 == 0 else len(q)
    if window <= polyorder:
        return None

    q_s = scipy.signal.savgol_filter(q, window, polyorder, axis=0, mode="interp")
    dq_s = scipy.signal.savgol_filter(q, window, polyorder, deriv=1, delta=dt, axis=0, mode="interp")
    ddq_s = scipy.signal.savgol_filter(q, window, polyorder, deriv=2, delta=dt, axis=0, mode="interp")
    return q_s, dq_s, ddq_s, window, dt


def build_regressor_function(robotdef, cache_path, verbose):
    if cache_path.exists():
        code = cache_path.read_text(encoding="utf-8")
    else:
        model = sympybotics.RobotDynCode(robotdef, verbose=verbose)
        code = sympybotics.robotcodegen.robot_code_to_func(
            "python", model.H_code, "H", "franka_regressor", robotdef
        )
        cache_path.parent.mkdir(parents=True, exist_ok=True)
        cache_path.write_text(code, encoding="utf-8")

    namespace = {"math": math, "numpy": np, "zeros": np.zeros, "sign": np.sign}
    exec_(code, namespace, namespace)
    return namespace["franka_regressor"]


def choose_samples(n_samples, max_samples):
    if max_samples <= 0 or n_samples <= max_samples:
        return np.arange(n_samples)
    return np.linspace(0, n_samples - 1, max_samples, dtype=int)


def stack_regression(regressor, q, dq, ddq, tau, sample_indices):
    blocks = []
    targets = []
    for index in sample_indices:
        h = np.asarray(regressor(q[index], dq[index], ddq[index]), dtype=float).reshape(DOF, -1)
        blocks.append(h)
        targets.append(tau[index])
    return np.vstack(blocks), np.concatenate(targets)


def solve_ridge(y_matrix, tau_vector, ridge):
    if ridge <= 0:
        params, *_ = np.linalg.lstsq(y_matrix, tau_vector, rcond=None)
        return params
    gram = y_matrix.T @ y_matrix
    rhs = y_matrix.T @ tau_vector
    scale = np.trace(gram) / max(1, gram.shape[0])
    return np.linalg.solve(gram + ridge * scale * np.eye(gram.shape[0]), rhs)


def metrics(y_matrix, tau_vector, params):
    pred = y_matrix @ params
    err = pred - tau_vector
    rmse = float(np.sqrt(np.mean(err**2)))
    mae = float(np.mean(np.abs(err)))
    max_abs = float(np.max(np.abs(err)))
    denom = float(np.sqrt(np.mean((tau_vector - np.mean(tau_vector)) ** 2)))
    nrmse = rmse / denom if denom > 0 else float("nan")
    return {"rmse_nm": rmse, "mae_nm": mae, "max_abs_nm": max_abs, "nrmse": nrmse}


def joint_metrics(y_matrix, tau_vector, params):
    pred = (y_matrix @ params).reshape(-1, DOF)
    tau = tau_vector.reshape(-1, DOF)
    err = pred - tau
    return [
        {
            "joint": index,
            "rmse_nm": float(np.sqrt(np.mean(err[:, index] ** 2))),
            "mae_nm": float(np.mean(np.abs(err[:, index]))),
            "max_abs_nm": float(np.max(np.abs(err[:, index]))),
        }
        for index in range(DOF)
    ]


def main():
    parser = argparse.ArgumentParser(description="Identify a Franka Panda dynamics model with SymPyBotics.")
    parser.add_argument("--csv", type=Path, default=Path("examples/franka_letters/data/dynamics_calibration.csv"))
    parser.add_argument("--out", type=Path, default=Path("examples/franka_letters/data/dynamics_model"))
    parser.add_argument("--max-samples", type=int, default=2500)
    parser.add_argument("--test-ratio", type=float, default=0.2)
    parser.add_argument("--ridge", type=float, default=1e-8)
    parser.add_argument("--smooth-window-s", type=float, default=0.041)
    parser.add_argument("--polyorder", type=int, default=3)
    parser.add_argument("--raw-ddq", action="store_true", help="Use logged finite-difference dq/ddq directly.")
    parser.add_argument("--no-friction", action="store_true")
    parser.add_argument("--verbose", action="store_true")
    args = parser.parse_args()

    time, q, dq, ddq, tau = load_log(args.csv)
    smoothing = None
    if not args.raw_ddq:
        smoothing = smooth_kinematics(time, q, args.smooth_window_s, args.polyorder)
        if smoothing is not None:
            q, dq, ddq, window, dt = smoothing

    robotdef = make_franka_robotdef(friction=not args.no_friction)
    args.out.mkdir(parents=True, exist_ok=True)
    regressor = build_regressor_function(robotdef, args.out / "franka_regressor.py", args.verbose)

    indices = choose_samples(len(q), args.max_samples)
    split = int(round(len(indices) * (1.0 - args.test_ratio)))
    train_indices = indices[:split]
    test_indices = indices[split:]

    train_y, train_tau = stack_regression(regressor, q, dq, ddq, tau, train_indices)
    test_y, test_tau = stack_regression(regressor, q, dq, ddq, tau, test_indices)
    params = solve_ridge(train_y, train_tau, args.ridge)

    param_names = [str(param) for param in robotdef.dynparms()]
    param_table = pd.DataFrame({"parameter": param_names, "value": params})
    param_table.to_csv(args.out / "franka_identified_parameters.csv", index=False)

    report = {
        "csv": str(args.csv),
        "valid_samples": int(len(q)),
        "used_samples": int(len(indices)),
        "train_samples": int(len(train_indices)),
        "test_samples": int(len(test_indices)),
        "dh_convention": robotdef.dh_convention,
        "friction_model": sorted(robotdef.frictionmodel) if robotdef.frictionmodel else [],
        "parameter_count": len(param_names),
        "ridge": args.ridge,
        "kinematics_source": "logged dq/ddq_est" if args.raw_ddq or smoothing is None else "savgol(q)",
        "savgol_window_samples": int(window) if smoothing is not None else None,
        "sample_time_s": float(dt) if smoothing is not None else float(np.median(np.diff(time))),
        "train_metrics": metrics(train_y, train_tau, params),
        "test_metrics": metrics(test_y, test_tau, params),
        "test_metrics_by_joint": joint_metrics(test_y, test_tau, params),
        "matrix_rank_train": int(np.linalg.matrix_rank(train_y)),
        "condition_number_train": float(np.linalg.cond(train_y)),
    }
    (args.out / "franka_dynamics_report.json").write_text(json.dumps(report, indent=2), encoding="utf-8")

    md = [
        "# Franka Panda Dynamics Identification",
        "",
        f"CSV: `{args.csv}`",
        f"Valid samples: {report['valid_samples']}",
        f"Used samples: {report['used_samples']} ({report['train_samples']} train, {report['test_samples']} test)",
        f"Model: 7-DOF modified DH rigid-body regressor, friction={report['friction_model']}",
        f"Kinematics source: {report['kinematics_source']}",
        f"Parameter count: {report['parameter_count']}",
        f"Train regressor rank: {report['matrix_rank_train']} / {report['parameter_count']}",
        "",
        "## Metrics",
        "",
        f"Train RMSE: {report['train_metrics']['rmse_nm']:.6g} Nm, NRMSE: {report['train_metrics']['nrmse']:.6g}",
        f"Test RMSE: {report['test_metrics']['rmse_nm']:.6g} Nm, NRMSE: {report['test_metrics']['nrmse']:.6g}",
        f"Test MAE: {report['test_metrics']['mae_nm']:.6g} Nm, Max abs: {report['test_metrics']['max_abs_nm']:.6g} Nm",
        "",
        "## Test RMSE by Joint",
        "",
        "| Joint | RMSE (Nm) | MAE (Nm) | Max abs (Nm) |",
        "| --- | ---: | ---: | ---: |",
    ]
    for item in report["test_metrics_by_joint"]:
        md.append(
            f"| {item['joint']} | {item['rmse_nm']:.6g} | {item['mae_nm']:.6g} | {item['max_abs_nm']:.6g} |"
        )
    md += [
        "",
        "## Files",
        "",
        "- `franka_identified_parameters.csv`: identified barycentric and friction parameters",
        "- `franka_regressor.py`: generated SymPyBotics regressor function",
        "- `franka_dynamics_report.json`: machine-readable summary",
    ]
    (args.out / "franka_dynamics_report.md").write_text("\n".join(md) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()