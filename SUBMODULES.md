# Submodule Notes

This workspace uses Git submodules for several external projects. Most submodules can be initialized with the usual command:

```powershell
git submodule update --init --recursive
```

## `utils/topp` and Git LFS data

`utils/topp` points to `git@github.com:TOPP-THU/topp.git`. That repository tracks many test CSV files with Git LFS, including large CNC test data. For normal development in this workspace, those full CSV payloads are not required.

To avoid downloading large LFS objects when initializing only `utils/topp`, use `GIT_LFS_SKIP_SMUDGE=1`.

PowerShell:

```powershell
$env:GIT_LFS_SKIP_SMUDGE = "1"
git submodule update --init --checkout utils/topp
Remove-Item Env:\GIT_LFS_SKIP_SMUDGE
```

Bash:

```bash
GIT_LFS_SKIP_SMUDGE=1 git submodule update --init --checkout utils/topp
```

After this, the large CSV files remain as small Git LFS pointer files in the working tree. If the full test data is ever needed, fetch it explicitly inside the submodule:

```powershell
git -C utils/topp lfs pull
```

To make an existing local `utils/topp` checkout avoid future automatic LFS downloads:

```powershell
git -C utils/topp lfs install --local --skip-smudge
```

If an interrupted checkout already downloaded large LFS objects, they can be removed from the local submodule cache without changing any commit:

```powershell
$lfsObjects = ".git/modules/utils/topp/lfs/objects"
if (Test-Path $lfsObjects) { Remove-Item -Recurse -Force "$lfsObjects\*" }
```
