import os

import plotly.graph_objects as go

data = {
    "Python": {
        "Walk": {
            "Linux": {
                "usr": {
                    "os.walk": 4.09372,
                    "os.walk (stat)": 11.3418,
                    "Walk.iter": 0.925864,
                    "Walk(Ext).iter": 0.96183,
                    "Walk.collect": 1.47056,
                    "Walk(Ext).collect": 1.36103,
                },
                "linux-5.9": {
                    "os.walk": 0.448813,
                    "os.walk (stat)": 1.64711,
                    "Walk.iter": 0.149128,
                    "Walk(Ext).iter": 0.143961,
                    "Walk.collect": 0.213981,
                    "Walk(Ext).collect": 0.211384,
                },
            },
            "Windows": {
                "Windows": {
                    "os.walk": 99.0955,
                    "os.walk (stat)": 238.835,
                    "Walk.iter": 10.0431,
                    "Walk(Ext).iter": 10.007,
                    "Walk.collect": 11.8813,
                    "Walk(Ext).collect": 11.8674,
                },
                "linux-5.9": {
                    "os.walk": 2.29283,
                    "os.walk (stat)": 17.6911,
                    "Walk.iter": 0.247534,
                    "Walk(Ext).iter": 0.250716,
                    "Walk.collect": 0.386362,
                    "Walk(Ext).collect": 0.39245,
                },
            },
        },
        "Scandir": {
            "Linux": {
                "usr": {
                    "scantree (os.scandir)": 8.75475,
                    "Scandir.iter": 1.37387,
                    "Scandir(Ext).iter": 1.87683,
                    "Scandir.collect": 2.16722,
                    "Scandir(Ext).collect": 2.92552,
                },
                "linux-5.9": {
                    "scantree (os.scandir)": 1.4078,
                    "Scandir.iter": 0.251858,
                    "Scandir(Ext).iter": 0.339001,
                    "Scandir.collect": 0.298834,
                    "Scandir(Ext).collect": 0.431882,
                },
            },
            "Windows": {
                "Windows": {
                    "scantree (os.scandir)": 66.8014,
                    "Scandir.iter": 10.1068,
                    "Scandir(Ext).iter": 37.7527,
                    "Scandir.collect": 11.3297,
                    "Scandir(Ext).collect": 38.5138,
                },
                "linux-5.9": {
                    "scantree (os.scandir)": 1.96715,
                    "Scandir.iter": 0.26433,
                    "Scandir(Ext).iter": 1.86403,
                    "Scandir.collect": 0.375734,
                    "Scandir(Ext).collect": 2.08924,
                },
            },
        },
    },
    "Rust": {
        "Walk": {
            "Linux": {
                "usr": {
                    "walkdir.WalkDir": 0.688,
                    "Walk.collect": 0.431,
                    "Walk(Ext).collect": 0.429,
                },
                "linux-5.9": {
                    "walkdir.WalkDir": 0.090843,
                    "Walk.collect": 0.059257,
                    "Walk(Ext).collect": 0.058337,
                },
            },
            "Windows": {
                "Windows": {
                    "walkdir.WalkDir": 15.257,
                    "Walk.collect": 3.046,
                    "Walk(Ext).collect": 2.961,
                },
                "linux-5.9": {
                    "walkdir.WalkDir": 0.484,
                    "Walk.collect": 0.1,
                    "Walk(Ext).collect": 0.099,
                },
            },
        },
        "Scandir": {
            "Linux": {
                "usr": {
                    "scan_dir.ScanDir": 1.4842,
                    "Scandir.collect": 0.63499,
                    "Scandir(Ext).collect": 0.8931,
                },
                "linux-5.9": {
                    "scan_dir.ScanDir": 0.20626,
                    "Scandir.collect": 0.071707,
                    "Scandir(Ext).collect": 0.11474,
                },
            },
            "Windows": {
                "Windows": {
                    "scan_dir.ScanDir": 15.13,
                    "Scandir.collect": 2.784,
                    "Scandir(Ext).collect": 10.162,
                },
                "linux-5.9": {
                    "scan_dir.ScanDir": 0.436,
                    "Scandir.collect": 0.086,
                    "Scandir(Ext).collect": 0.779,
                },
            },
        },
    },
}

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
                fig.write_image(
                    f"{dirName}/{osName.lower()}_{methodGroup.lower()}_{path.lower()}.png"
                )
