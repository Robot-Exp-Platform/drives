# Franka Panda Dynamics Identification

CSV: `examples\franka_letters\data\dynamics_calibration.csv`
Valid samples: 46166
Used samples: 2500 (2000 train, 500 test)
Model: 7-DOF modified DH rigid-body regressor, friction=['Coulomb', 'offset', 'viscous']
Kinematics source: savgol(q)
Parameter count: 91
Train regressor rank: 64 / 91

## Metrics

Train RMSE: 0.44831 Nm, NRMSE: 0.0505128
Test RMSE: 0.470862 Nm, NRMSE: 0.0466639
Test MAE: 0.313876 Nm, Max abs: 2.35591 Nm

## Test RMSE by Joint

| Joint | RMSE (Nm) | MAE (Nm) | Max abs (Nm) |
| --- | ---: | ---: | ---: |
| 0 | 0.814578 | 0.669682 | 2.24831 |
| 1 | 0.504006 | 0.400118 | 1.61164 |
| 2 | 0.679357 | 0.535488 | 2.35591 |
| 3 | 0.337412 | 0.26744 | 1.05287 |
| 4 | 0.150331 | 0.122328 | 0.641656 |
| 5 | 0.167655 | 0.129631 | 0.561923 |
| 6 | 0.0912966 | 0.0724462 | 0.348587 |

## Files

- `franka_identified_parameters.csv`: identified barycentric and friction parameters
- `franka_regressor.py`: generated SymPyBotics regressor function
- `franka_dynamics_report.json`: machine-readable summary
