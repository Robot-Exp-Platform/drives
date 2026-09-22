from pathlib import Path
import argparse,hashlib,json,shutil,subprocess
parser=argparse.ArgumentParser(description="Reproduce isolated Jaka parser tests, not whole-driver validation")
parser.add_argument("--output",type=Path,required=True)
args=parser.parse_args()
root=Path(__file__).resolve().parents[2]
base=args.output.resolve();base.mkdir(parents=True,exist_ok=False)
repo=root/'libjaka-rs'
source=repo/'src/types/robot_type.rs'
fixed=source.read_text()
original=subprocess.check_output(['git','show','bda271ba7907d1f63ec2d67710b18b4486abc5de:src/types/robot_type.rs'],cwd=repo,text=True)
common_prefix='''#![feature(adt_const_params)]
#![allow(incomplete_features)]
extern crate self as robot_behavior;
#[path=__EXCEPTION_PATH__]
mod production_exception;
pub use production_exception::{RobotException, RobotResult};
#[allow(dead_code)]
mod production_parser {
use std::marker::ConstParamTy;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use crate::{RobotException, RobotResult};
'''
common_prefix=common_prefix.replace('__EXCEPTION_PATH__',json.dumps(str(root/'robot_behavior/src/exception.rs')))
def span(s,start,end): return s[s.index(start):s.index(end,s.index(start))]
def selected(s):
 return ''.join([
  span(s,'pub trait CommandSerde','pub enum ErrorCode'),
  span(s,'#[derive(Serialize, Deserialize)]\npub struct DefaultState','// power on'),
  span(s,'pub type PowerOnResponse','// power off'),
  span(s,'pub type ServoMoveRequest','// servo j'),
  span(s,'impl<const C: Command, D> From<D>','#[cfg(test)]\nmod tests'),
 ])
tests=fixed[fixed.index('#[cfg(test)]\nmod tests'):]
for flavor,text in [('fixed',fixed),('baseline',original)]:
 pkg=base/flavor;pkg.mkdir(exist_ok=True)
 (pkg/'Cargo.toml').write_text(f'''[package]
name = "jaka-parser-{flavor}"
version = "0.0.0"
edition = "2024"
[workspace]
[features]
to_py = []
[lib]
path = "lib.rs"
[dependencies]
serde = {{version="1.0.228",features=["derive"]}}
serde_json = "1.0.145"
thiserror = "2.0.17"
anyhow = "1.0.100"
''')
 (pkg/'lib.rs').write_text(common_prefix+selected(text)+tests+'\n}\n')
manifest={
 'scope':'Exact source spans of production CommandSerde, Request/Response types and impls; current four tests. Error type is included from production exception.rs. Does not compile libjaka crate, robot_behavior traits or hardware.',
 'source':str(source),'fixed_sha256':hashlib.sha256(fixed.encode()).hexdigest(),'baseline_commit':'bda271ba7907d1f63ec2d67710b18b4486abc5de',
 'baseline_sha256':hashlib.sha256(original.encode()).hexdigest(),
 'exception_path':str(root/'robot_behavior/src/exception.rs'),
 'tests_source':'Current uncommitted libjaka src/types/robot_type.rs tests module; identical tests compiled against baseline/fixed method spans.'
}
(base/'provenance.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps(manifest,indent=2))

results={}
for flavor,expected in [('fixed',0),('baseline',101)]:
 pkg=base/flavor
 shutil.copyfile(Path(__file__).parent/'parser-locks'/f'{flavor}.lock',pkg/'Cargo.lock')
 with (base/f'{flavor}.log').open('w') as log:
  run=subprocess.run(['cargo','test','--locked','--offline'],cwd=pkg,stdout=log,stderr=subprocess.STDOUT)
 results[flavor]={'exit_code':run.returncode,'expected_exit_code':expected,'log':str(base/f'{flavor}.log')}
 print(flavor,results[flavor])
 summary = 'test result: ok. 4 passed; 0 failed' if flavor == 'fixed' else 'test result: FAILED. 2 passed; 2 failed'
 if run.returncode!=expected or summary not in (base/f'{flavor}.log').read_text():
  raise SystemExit(f'Unexpected {flavor} result; inspect the log, do not infer success')
(base/'results.json').write_text(json.dumps(results,indent=2)+'\n')
