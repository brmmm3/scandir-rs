# -*- mode: python ; coding: utf-8 -*-

block_cipher = None


a = Analysis(['fastproperties.py'],
             pathex=[],
             binaries=[],
             datas=[],
             hiddenimports=[],
             hookspath=[],
             runtime_hooks=[],
             excludes=['_bz2', '_ctypes', '_hashlib', '_lzma', '_socket', '_ssl', 'pyexpat', 'numpy', 'pytz'],
             win_no_prefer_redirects=False,
             win_private_assemblies=False,
             cipher=block_cipher,
             noarchive=False)

a.datas = [entry for entry in a.datas if not entry[0].startswith("lib2to3")]
a.datas = [entry for entry in a.datas
           if "tzdata" not in entry[0] and "email" not in entry[0] and "unittest" not in entry[0]
           and "msgs" not in entry[0] and "encoding" not in entry[0] and "README" not in entry[0]
           and "tai-ku" not in entry[0]]
a.datas.append(("fastproperties.ico", "fastproperties.ico", "DATA"))
a.binaries = [entry for entry in a.binaries if not entry[0].startswith("libopenblas")]

pyz = PYZ(a.pure, a.zipped_data,
          cipher=block_cipher)

exe = EXE(pyz,
          a.scripts,
          a.binaries,
          a.zipfiles,
          a.datas,
          [],
          name='fastproperties',
          debug=False,
          bootloader_ignore_signals=False,
          strip=False,
          upx=True,
          upx_exclude=[],
          runtime_tmpdir=None,
          console=False,
          icon="fastproperties.ico")
