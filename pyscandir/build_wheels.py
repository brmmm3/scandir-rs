import os
import sys
import subprocess
import concurrent.futures
from typing import List


def Run(args: List[str]) -> subprocess.CompletedProcess:
    print("RUN:", " ".join(args))
    if os.name == "nt":
        return subprocess.run(args, shell=True, capture_output=True)
    return subprocess.run(" ".join(args), shell=True, capture_output=True)


def ShowResult(title: str, prc: subprocess.CompletedProcess):
    stdout = prc.stdout.decode("utf-8")
    stderr = prc.stderr.decode("utf-8")
    if prc.returncode != 0:
        print(f"'{title}' failed with error code {prc.returncode}")
        print(stderr)
    elif not stdout:
        stdout = stderr
    return stdout, prc.returncode


def BuildWheel(
    versions_dir: str, version: str, python_exe: str, features: str, bDebug: bool
) -> int:
    print(f"Building wheel for Python version {version}...")
    python_path = f"{versions_dir}/{version}/{python_exe}"
    cmd = ["maturin", "build", "--strip", "-i", python_path]
    if not bDebug:
        cmd.insert(2, "--release")
    if features:
        cmd.extend(["--", "--features", f'"{features.replace(",", " ")}"'])
    maturin_build = Run(cmd)
    stdout, returncode = ShowResult("maturin build", maturin_build)
    if returncode != 0:
        return returncode
    builtWheel = [
        line for line in stdout.splitlines() if "Built wheel for CPython" in line
    ]
    if not builtWheel:
        print("No wheel built!")
        print(stdout)
        return 1
    wheel_path = builtWheel[0].split(" to ")[1]

    upgrade_pip = Run([python_path, "-m", "pip", "install", "-U", "pip"])
    stdout, returncode = ShowResult("pip install -U pip", upgrade_pip)
    if returncode != 0:
        return returncode

    upgrade_pytest = Run([python_path, "-m", "pip", "install", "-U", "pytest"])
    stdout, returncode = ShowResult("pip install -U pytest", upgrade_pytest)
    if returncode != 0:
        return returncode

    install_wheel = Run(
        [python_path, "-m", "pip", "install", "--force-reinstall", wheel_path]
    )
    stdout, returncode = ShowResult("install wheel", install_wheel)
    if returncode != 0:
        return returncode

    run_pytest = Run([python_path, "-m", "pytest"])
    stdout, returncode = ShowResult("pytest", run_pytest)
    print(stdout)
    if returncode != 0 and returncode != 5:
        return returncode
    return 0


if __name__ == "__main__":
    versions = [sys.version.split()[0]]
    if "--versions" in sys.argv:
        versions = sys.argv[sys.argv.index("--versions") + 1].split(",")
    if versions == ["*"]:
        pyenv_versions = Run(["pyenv", "versions"])
        stdout, returncode = ShowResult("pyenv versions", pyenv_versions)
        if pyenv_versions.returncode != 0:
            sys.exit(1)
        versions = [
            version.lstrip("*").strip().split()[0]
            for version in stdout.splitlines()
            if "system" not in version and " 2.7." not in version
        ]
    if not versions:
        print("No versions to build Python wheel!")
        sys.exit(1)
    features = None
    if "--features" in sys.argv:
        features = sys.argv[sys.argv.index("--features") + 1]
    bDebug = "--debug" in sys.argv

    print(f"Building wheel for Python versions {bDebug=}:")
    print("\n".join(versions))

    python_path = Run(["pyenv", "which", "python"])
    if python_path.returncode != 0:
        print(f"'pyenv which python' failed with error code {python_path.returncode}")
        print(python_path.stderr.decode("utf-8"))
        sys.exit(1)

    versions_dir = (
        python_path.stdout.decode("utf-8").rsplit("versions", 1)[0] + "versions"
    )

    python_exe = "python.exe" if os.name == "nt" else "bin/python"

    futures = {}
    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as executor:
        for version in versions:
            futures[version] = executor.submit(
                BuildWheel, versions_dir, version, python_exe, features, bDebug
            )
        for version, future in futures.items():
            returncode = future.result()
            if returncode != 0:
                print(
                    f"Building wheel for Python version {version} failed with error code {returncode}!"
                )
