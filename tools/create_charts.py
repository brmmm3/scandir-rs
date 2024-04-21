import os
import json

import plotly.graph_objects as go

data = {
    "Python": {
        "Walk": {
            "Linux": {
                "linux-5.9": {
                    "os.walk": 0.440817,
                    "Walk.iter": 0.133679,
                    "Walk.collect": 0.197038,
                    "os.walk(Ext)": 1.61088,
                    "Walk.iter(Ext)": 0.133556,
                    "Walk.collect(Ext)": 0.191944,
                },
                "usr": {
                    "os.walk": 3.94502,
                    "Walk.iter": 0.80265,
                    "Walk.collect": 1.34461,
                    "os.walk(Ext)": 10.7779,
                    "Walk.iter(Ext)": 0.827304,
                    "Walk.collect(Ext)": 1.33137,
                },
            },
            "Windows": {
                "linux-5.9": {
                    "os.walk": 2.29283,
                    "Walk.iter": 0.247534,
                    "Walk.collect": 0.386362,
                    "os.walk(Ext)": 17.6911,
                    "Walk.iter(Ext)": 0.250716,
                    "Walk.collect(Ext)": 0.39245,
                },
                "Windows": {
                    "os.walk": 99.0955,
                    "Walk.iter": 10.0431,
                    "Walk.collect": 11.8813,
                    "os.walk(Ext)": 238.835,
                    "Walk.iter(Ext)": 10.007,
                    "Walk.collect(Ext)": 11.8674,
                },
            },
        },
        "Scandir": {
            "Linux": {
                "linux-5.9": {
                    "scantree (os.scandir)": 1.31862,
                    "Scandir.iter": 0.237867,
                    "Scandir.collect": 0.271947,
                    "Scandir.iter(Ext)": 0.320545,
                    "Scandir.collect(Ext)": 0.380465,
                },
                "usr": {
                    "scantree (os.scandir)": 8.25362,
                    "Scandir.iter": 1.27802,
                    "Scandir.collect": 2.01097,
                    "Scandir.iter(Ext)": 1.75471,
                    "Scandir.collect(Ext)": 2.58515,
                },
            },
            "Windows": {
                "linux-5.9": {
                    "scantree (os.scandir)": 1.96715,
                    "Scandir.iter": 0.26433,
                    "Scandir.collect": 0.375734,
                    "Scandir.iter(Ext)": 1.86403,
                    "Scandir.collect(Ext)": 2.08924,
                },
                "Windows": {
                    "scantree (os.scandir)": 66.8014,
                    "Scandir.iter": 10.1068,
                    "Scandir.collect": 11.3297,
                    "Scandir.iter(Ext)": 37.7527,
                    "Scandir.collect(Ext)": 38.5138,
                },
            },
        },
    },
    "Rust": {
        "Walk": {
            "Linux": {
                "linux-5.9": {
                    "walkdir.WalkDir": 0.082,
                    "Walk.collect": 0.056,
                    "walkdir.WalkDir(Ext)": 0.462,
                    "Walk.collect(Ext)": 0.055,
                },
                "usr": {
                    "walkdir.WalkDir": 0.671,
                    "Walk.collect": 0.405,
                    "walkdir.WalkDir(Ext)": 2.829,
                    "Walk.collect(Ext)": 0.404,
                },
            },
            "Windows": {
                "linux-5.9": {
                    "walkdir.WalkDir": 0.456,
                    "Walk.collect": 0.1,
                    "walkdir.WalkDir(Ext)": 4.343,
                    "Walk.collect(Ext)": 0.103,
                },
                "Windows": {
                    "walkdir.WalkDir": 15.546,
                    "Walk.collect": 3.454,
                    "walkdir.WalkDir(Ext)": 50.366,
                    "Walk.collect(Ext)": 3.459,
                },
            },
        },
        "Scandir": {
            "Linux": {
                "linux-5.9": {
                    "scan_dir.ScanDir": 0.199,
                    "Scandir.collect": 0.073,
                    "scan_dir.ScanDir(Ext)": 0.383,
                    "Scandir.collect(Ext)": 0.116,
                },
                "usr": {
                    "scan_dir.ScanDir": 1.474,
                    "Scandir.collect": 0.615,
                    "scan_dir.ScanDir(Ext)": 2.575,
                    "Scandir.collect(Ext)": 0.822,
                },
            },
            "Windows": {
                "linux-5.9": {
                    "scan_dir.ScanDir": 0.456,
                    "Scandir.collect": 0.107,
                    "scan_dir.ScanDir(Ext)": 7.483,
                    "Scandir.collect(Ext)": 0.864,
                },
                "Windows": {
                    "scan_dir.ScanDir": 16.818,
                    "Scandir.collect": 2.999,
                    "scan_dir.ScanDir(Ext)": 47.740,
                    "Scandir.collect(Ext)": 10.632,
                },
            },
        },
    },
}


def UpdateData(lang: str, bs: str, path: str, newData: dict):
    for method in ("Walk", "Scandir"):
        d = data[lang][method][bs][path]
        for key in tuple(d):
            d[key] = newData[key]


py_nt_linux = json.loads(
    open("tools/benchmark_results_nt_linux-5.9_python.json").read()
)
py_nt_windows = json.loads(
    open("tools/benchmark_results_nt_Windows_python.json").read()
)
py_linux_linux = json.loads(
    open("tools/benchmark_results_posix_linux-5.9_python.json").read()
)
py_linux_usr = json.loads(open("tools/benchmark_results_posix_usr_python.json").read())

UpdateData("Python", "Windows", "linux-5.9", py_nt_linux)
UpdateData("Python", "Windows", "Windows", py_nt_windows)
UpdateData("Python", "Linux", "linux-5.9", py_linux_linux)
UpdateData("Python", "Linux", "usr", py_linux_usr)


for lang, langData in data.items():
    baseDir = "pyscandir" if lang == "Python" else "scandir"
    dirName = f"{baseDir}/doc/images"
    if not os.path.exists(dirName):
        os.makedirs(dirName)
    for methodGroup, methodData in langData.items():
        for osName, osData in methodData.items():
            for path, pathData in osData.items():
                methods = list(pathData.keys())
                fig = go.Figure(
                    data=[
                        go.Bar(
                            name=method,
                            x=["" * len(methods)],
                            y=[dt],
                            text=f"{dt:.2f}s",
                            textposition="auto",
                        )
                        for method, dt in pathData.items()
                    ]
                )
                fig.update_layout(
                    barmode="group",
                    xaxis_title="Method",
                    yaxis_title="Time [s]",
                )
                pathName = f"{dirName}/{osName.lower()}_{methodGroup.lower()}_{path.lower()}.png"
                print(pathName)
                fig.write_image(pathName)
