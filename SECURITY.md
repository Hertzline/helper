# Security Policy

Report vulnerabilities privately as described in the organization
[SECURITY.md](https://github.com/Hertzline/.github/blob/main/SECURITY.md).

Do not open public issues for undisclosed vulnerabilities.

## Helper surface

This repository owns the privileged broker. It must accept only a fixed
allow-list of named operations. No arbitrary shell, no generic "run command".

Allow-list (HZ-019): `ping`, `probe_elevate_path`, `restore_all`,
`create_restore_point` (Windows System Restore via SrClient.dll
`SRSetRestorePointW`; not possible on Linux/macOS).
