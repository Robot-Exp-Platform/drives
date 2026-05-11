//! Run a preplanned joint trajectory on a real Franka and log dynamics data.
//!
//! The motion command is generated with `move_with_closure`. Logging reads the
//! full Franka `RobotState` through a `read_state` helper, not the `ArmState`
//! passed into the realtime closure. Franka state exposes measured `q`, `dq`,
//! and `tau_j`; measured acceleration and jerk are not present in the state, so
//! this example records finite-difference estimates from measured `dq`.

use anyhow::Result;
use franka_rust::{
    FrankaEmika,
    types::{
        robot_state::{RobotState, RobotStateInter},
        robot_types::SetCollisionBehaviorData,
    },
};
use robot_behavior::{MotionType, RobotResult, behavior::*};
use serde_json::from_reader;
use std::{
    env,
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::Path,
    sync::{Arc, RwLock},
    time::Duration,
};

const DOF: usize = 7;
const DEFAULT_TRAJ_FILE: &str = "data/joint_lissajous_traj.json";
const DEFAULT_LOG_FILE: &str = "data/dynamics_calibration.csv";

struct FrankaStateReader {
    robot_state: Arc<RwLock<RobotStateInter>>,
}

impl FrankaStateReader {
    fn read_state(&mut self) -> RobotResult<RobotState> {
        let state = self.robot_state.read().unwrap();
        Ok((*state).into())
    }
}

struct DynamicsLogger {
    writer: BufWriter<File>,
    last_t: Option<f64>,
    last_dq: Option<[f64; DOF]>,
    last_ddq: Option<[f64; DOF]>,
}

impl DynamicsLogger {
    fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        let mut writer = BufWriter::new(File::create(path)?);
        write!(writer, "t_s")?;
        write_vec_header(&mut writer, "q")?;
        write_vec_header(&mut writer, "dq")?;
        write_vec_header(&mut writer, "ddq_est")?;
        write_vec_header(&mut writer, "dddq_est")?;
        write_vec_header(&mut writer, "tau_j")?;
        writeln!(writer)?;

        Ok(Self { writer, last_t: None, last_dq: None, last_ddq: None })
    }

    fn write_state(&mut self, t: f64, state: &RobotState) -> std::io::Result<()> {
        let dt = self.last_t.map(|last_t| t - last_t).filter(|dt| *dt > 0.0);
        let ddq = match (dt, self.last_dq) {
            (Some(dt), Some(last_dq)) => Some(diff_array(&state.dq, &last_dq, dt)),
            _ => None,
        };
        let dddq = match (dt, ddq, self.last_ddq) {
            (Some(dt), Some(ddq), Some(last_ddq)) => Some(diff_array(&ddq, &last_ddq, dt)),
            _ => None,
        };

        write!(self.writer, "{t:.9}")?;
        write_array(&mut self.writer, &state.q)?;
        write_array(&mut self.writer, &state.dq)?;
        write_optional_array(&mut self.writer, ddq.as_ref())?;
        write_optional_array(&mut self.writer, dddq.as_ref())?;
        write_array(&mut self.writer, &state.tau_j)?;
        writeln!(self.writer)?;

        self.last_t = Some(t);
        self.last_dq = Some(state.dq);
        if let Some(ddq) = ddq {
            self.last_ddq = Some(ddq);
        }
        Ok(())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

fn main() -> Result<()> {
    let host = env::var("FRANKA_HOST").unwrap_or_else(|_| "172.16.0.3".to_string());
    let traj_file =
        env::var("FRANKA_DYNAMICS_TRAJ_FILE").unwrap_or_else(|_| DEFAULT_TRAJ_FILE.into());
    let log_file = env::var("FRANKA_DYNAMICS_LOG_FILE").unwrap_or_else(|_| DEFAULT_LOG_FILE.into());
    let path = load_joint_path(&traj_file)?;

    let mut robot = FrankaEmika::new(&host);
    robot.set_default_behavior()?;
    robot.set_collision_behavior(SetCollisionBehaviorData {
        lower_torque_thresholds_acceleration: [20., 20., 18., 18., 16., 14., 12.],
        upper_torque_thresholds_acceleration: [20., 20., 18., 18., 16., 14., 12.],
        lower_torque_thresholds_nominal: [20., 20., 18., 18., 16., 14., 12.],
        upper_torque_thresholds_nominal: [20., 20., 18., 18., 16., 14., 12.],
        lower_force_thresholds_acceleration: [20., 20., 20., 25., 25., 25.],
        upper_force_thresholds_acceleration: [20., 20., 20., 25., 25., 25.],
        lower_force_thresholds_nominal: [20., 20., 20., 25., 25., 25.],
        upper_force_thresholds_nominal: [20., 20., 20., 25., 25., 25.],
    })?;

    robot.move_joint(&path[0])?;

    let state_handle = robot.robot_impl.robot_state.clone();
    let mut state_reader = FrankaStateReader { robot_state: state_handle };
    let mut logger = DynamicsLogger::new(&log_file)?;
    let mut elapsed = Duration::ZERO;
    let mut sample_idx = 0usize;
    let sample_count = path.len();
    let planned_duration = (sample_count.saturating_sub(1)) as f64 * FrankaEmika::CONTROL_PERIOD;

    println!(
        "[dynamics-calibration] running on {host}, samples={sample_count}, planned_duration={planned_duration:.3}s, traj={traj_file}, log={log_file}",
    );
    println!(
        "[dynamics-calibration] make sure the workspace is clear and the user-stop is reachable!"
    );

    robot.move_with_closure(move |_, dt| {
        elapsed += dt;
        let t = elapsed.as_secs_f64();

        if let Ok(state) = state_reader.read_state() {
            if let Err(err) = logger.write_state(t, &state) {
                eprintln!("[dynamics-calibration] failed to write sample: {err}");
            }
        }

        let target = path
            .get(sample_idx)
            .copied()
            .unwrap_or_else(|| *path.last().expect("path is checked non-empty"));
        sample_idx = sample_idx.saturating_add(1);
        let finished = sample_idx >= sample_count;
        if finished {
            let _ = logger.flush();
        }

        (MotionType::Joint(target), finished)
    })?;

    robot.waiting_for_finish()?;
    println!("[dynamics-calibration] finished, log={log_file}");

    Ok(())
}

fn load_joint_path(path: impl AsRef<Path>) -> Result<Vec<[f64; DOF]>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let motions: Vec<MotionType<DOF>> = from_reader(reader)?;
    let mut joints = Vec::with_capacity(motions.len());
    for motion in motions {
        match motion {
            MotionType::Joint(q) => joints.push(q),
            other => {
                anyhow::bail!("dynamics calibration only supports Joint trajectory, got {other:?}")
            }
        }
    }
    if joints.is_empty() {
        anyhow::bail!("dynamics calibration trajectory is empty");
    }
    Ok(joints)
}

fn diff_array(current: &[f64; DOF], last: &[f64; DOF], dt: f64) -> [f64; DOF] {
    let mut out = [0.0; DOF];
    for i in 0..DOF {
        out[i] = (current[i] - last[i]) / dt;
    }
    out
}

fn write_vec_header(writer: &mut impl Write, name: &str) -> std::io::Result<()> {
    for i in 0..DOF {
        write!(writer, ",{name}{i}")?;
    }
    Ok(())
}

fn write_array(writer: &mut impl Write, values: &[f64; DOF]) -> std::io::Result<()> {
    for value in values {
        write!(writer, ",{value:.12}")?;
    }
    Ok(())
}

fn write_optional_array(
    writer: &mut impl Write,
    values: Option<&[f64; DOF]>,
) -> std::io::Result<()> {
    match values {
        Some(values) => write_array(writer, values),
        None => {
            for _ in 0..DOF {
                write!(writer, ",")?;
            }
            Ok(())
        }
    }
}
