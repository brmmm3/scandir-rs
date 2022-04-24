#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import os
import sys
import time
import stat
from datetime import datetime
from tkinter import Tk, messagebox, BooleanVar, PhotoImage
from tkinter.ttk import Label, Checkbutton, Separator, Style

from scandir_rs import Count

KB = 1024
MB = KB * KB
GB = MB * KB


def GetFileIconWin(pathName):
    import io
    # noinspection PyUnresolvedReferences
    from win32com.shell import shell
    from PIL import Image, ImageTk
    import win32api
    import win32con
    import win32ui
    import win32gui

    SHGFI_ICON = 0x000000100
    SHGFI_ICONLOCATION = 0x000001000
    SHIL_SIZE = 0x00001  # Small icon

    ret, info = shell.SHGetFileInfo(
        pathName, 0, SHGFI_ICONLOCATION | SHGFI_ICON | SHIL_SIZE)
    hIcon, iIcon, dwAttr, name, typeName = info
    ico_x = win32api.GetSystemMetrics(win32con.SM_CXICON)
    hdc = win32ui.CreateDCFromHandle(win32gui.GetDC(0))
    hbmp = win32ui.CreateBitmap()
    hbmp.CreateCompatibleBitmap(hdc, ico_x, ico_x)
    hdc = hdc.CreateCompatibleDC()
    hdc.SelectObject(hbmp)
    hdc.DrawIcon((0, 0), hIcon)
    win32gui.DestroyIcon(hIcon)
    bmpinfo = hbmp.GetInfo()
    img = Image.frombuffer("RGBA", (bmpinfo["bmWidth"], bmpinfo["bmHeight"]),
                           hbmp.GetBitmapBits(True), "raw", "BGRA", 0, 1)
    img = img.resize((16, 16), Image.ANTIALIAS)
    buf = io.BytesIO()
    img.save(buf, format='GIF')
    return ImageTk.PhotoImage(img)


def GetFileOwnerWin(pathName):
    import win32security
    sid = win32security.GetFileSecurity(
        pathName, win32security.OWNER_SECURITY_INFORMATION).GetSecurityDescriptorOwner()
    name, domain, _ = win32security.LookupAccountSid(None, sid)
    return f"{domain}\\{name}"


def GetFileOwnerLin(pathName):
    from pwd import getpwuid
    return getpwuid(os.lstat(pathName).st_uid).pw_name


class FastProperties(Tk):

    def __init__(self, pathName: str):
        super().__init__()
        self.title("Fast Properties")
        self.pathName: str = pathName
        # Create main window
        self.configure(background=Style().lookup("CheckButton", "background"))
        self.lblRootDir = Label(self, text="Analyzing...", background="yellow")
        self.lblRootDir.grid(row=0, columnspan=3)
        if os.name == 'nt':
            fileIcon = GetFileIconWin(pathName)
            fileOwner = GetFileOwnerWin(pathName)
            self.iconbitmap(os.path.join(
                getattr(sys, "_MEIPASS", "."), "fastproperties.ico"))
        else:
            fileIcon = ""
            fileOwner = GetFileOwnerLin(pathName)
            image = PhotoImage(file=os.path.join(
                getattr(sys, "_MEIPASS", "."), 'fastproperties.gif'))
            self.iconphoto(False, image)
        self.addCell(1, fileIcon, os.path.basename(pathName))
        Separator(self, orient="horizontal").grid(
            row=2, columnspan=3, sticky="we")
        self.addCell(3, "Type:", self.getType(pathName))
        self.addCell(4, "Location:", os.path.dirname(pathName))
        self.addCell(5, "Owner:", fileOwner)
        self.lblSizeTitle, self.lblSize = self.addCell(6, "Size:")
        self.lblUsageTitle, self.lblUsage = self.addCell(7, "Size on disk:")
        self.lblStatsTitle, self.lblStats = self.addCell(8, "Contains:")
        Separator(self, orient="horizontal").grid(
            row=9, columnspan=3, sticky="we")
        stats = os.lstat(pathName)
        self.addCell(10, "Created:", datetime.fromtimestamp(
            stats.st_ctime).strftime("%A, %d. %B %Y, %H:%M:%S"))
        self.addCell(11, "Modified:", datetime.fromtimestamp(
            stats.st_mtime).strftime("%A, %d. %B %Y, %H:%M:%S"))
        self.addCell(12, "Accessed:", datetime.fromtimestamp(
            stats.st_atime).strftime("%A, %d. %B %Y, %H:%M:%S"))
        Separator(self, orient="horizontal").grid(
            row=13, columnspan=3, sticky="we")
        self.addCell(14, "Attributes:", "")
        if os.name == "nt":
            self.bReadOnly = self.addCheckButton(
                15, 0, "Read-only", stats.st_file_attributes & stat.FILE_ATTRIBUTE_READONLY)
            self.bArchive = self.addCheckButton(
                15, 2, "Archive", stats.st_file_attributes & stat.FILE_ATTRIBUTE_ARCHIVE)
            self.bHidden = self.addCheckButton(
                16, 0, "Hidden", stats.st_file_attributes & stat.FILE_ATTRIBUTE_HIDDEN)
            self.bCompressed = self.addCheckButton(
                16, 2, "Compressed", stats.st_file_attributes & stat.FILE_ATTRIBUTE_COMPRESSED)
            self.bEncrypted = self.addCheckButton(
                17, 0, "Encrypted", stats.st_file_attributes & stat.FILE_ATTRIBUTE_ENCRYPTED)
            self.bEncrypted = self.addCheckButton(
                17, 2, "Reparse Point", stats.st_file_attributes & stat.FILE_ATTRIBUTE_REPARSE_POINT)
        else:
            self.bReadOnly = self.addCheckButton(
                15, 0, "Set UID", stats.st_mode & stat.S_ISUID)
            self.bArchive = self.addCheckButton(
                15, 1, "Set GID", stats.st_mode & stat.S_ISGID)
            self.bArchive = self.addCheckButton(
                15, 2, "Sticky", stats.st_mode & stat.S_ISVTX)
            mode = stats.st_mode
            self.owner = self.addCell(16, "Owner:",
                                      f"{'r' if mode & stat.S_IRUSR else '-'}"
                                      f"{'w' if mode & stat.S_IWUSR else '-'}"
                                      f"{'x' if mode & stat.S_IXUSR else '-'}")
            self.group = self.addCell(17, "Group:",
                                      f"{'r' if mode & stat.S_IRGRP else '-'}"
                                      f"{'w' if mode & stat.S_IWGRP else '-'}"
                                      f"{'x' if mode & stat.S_IXGRP else '-'}")
            self.others = self.addCell(18, "Others:",
                                       f"{'r' if mode & stat.S_IROTH else '-'}"
                                       f"{'w' if mode & stat.S_IWOTH else '-'}"
                                       f"{'x' if mode & stat.S_IXOTH else '-'}")
        # Catch window deletion event
        self.protocol("WM_DELETE_WINDOW", self._delete_window)
        self.bind("<Destroy>", self._destroy)
        # Center main window
        posRight = self.winfo_screenwidth() // 2 - self.winfo_reqwidth() // 2
        posDown = self.winfo_screenheight() // 2 - self.winfo_reqheight() // 2
        self.geometry(f"+{posRight}+{posDown}")
        self.minsize(250, 160)
        # Start directory scanner
        self.startTime = time.time()
        self.timerId = None
        self.scanner = Count(pathName, extended=True)
        self.scanner.start()
        self.updater()

    def _delete_window(self):
        try:
            self.destroy()
        except:
            pass

    def _destroy(self, _):
        self.withdraw()
        if self.timerId is not None:
            self.after_cancel(self.timerId)
        if self.scanner is not None:
            self.scanner.stop()
            self.scanner = None
        sys.exit(0)

    def addCell(self, row: int, title, value: str = None):
        if isinstance(title, str):
            lblTitle = Label(self, text=title)
        else:
            lblTitle = Label(self, image=title)
            lblTitle.image = title
        lblTitle.grid(row=row, column=0, padx=10, sticky="nw")
        lblValue = Label(self, anchor="nw", justify="left", text=value)
        lblValue.grid(row=row, column=1, columnspan=2, padx=10, sticky="nw")
        return lblTitle, lblValue

    @staticmethod
    def addCheckButton(row, col, title, value):
        var = BooleanVar(value=bool(value))
        Checkbutton(text=title, variable=var, command=lambda: var.set(
            1 - var.get())).grid(row=row, column=col, sticky="nw")
        return var

    @staticmethod
    def getType(root):
        if os.path.ismount(root):
            return "Mount point"
        if os.path.islink(root):
            return f"Symbolic link\n-> {os.path.realpath(root)}"
        if os.path.isfile(root):
            return "File"
        if os.path.isdir(root):
            return "File folder"
        return "?"

    @staticmethod
    def formatByteSize(size: int):
        if size < KB:
            return f"{str(size)} Bytes"
        if size < MB:
            return f"{'{0:.2f}'.format(size / KB)} kB"
        if size < GB:
            return f"{'{0:.2f}'.format(size / MB)} MB"
        return f"{'{0:.2f}'.format(size / GB)} GB"

    @staticmethod
    def formatBigNumbers(nr: int):
        return str(nr) if nr < 1000 else f"{FastProperties.formatBigNumbers(nr // 1000)}.{str(nr)[-3:]}"

    def update(self):
        stats = self.scanner.results()
        self.lblSize.config(
            text=f"{self.formatByteSize(stats.size)} ({self.formatBigNumbers(stats.size)} bytes)")
        self.lblUsage.config(
            text=f"{self.formatByteSize(stats.usage)} ({self.formatBigNumbers(stats.usage)} bytes)")
        line1 = []
        line2 = []
        line3 = []
        if stats.files > 0:
            line1.append(f"{self.formatBigNumbers(stats.files)} Files")
        if stats.dirs > 0:
            line1.append(f"{self.formatBigNumbers(stats.dirs)} Folders")
        if stats.slinks > 0:
            line2.append(f"{self.formatBigNumbers(stats.slinks)} Symlinks")
        if stats.hlinks > 0:
            line2.append(f"{self.formatBigNumbers(stats.hlinks)} Hardlinks")
        if stats.devices > 0:
            line3.append(f"{self.formatBigNumbers(stats.devices)} Devices")
        if stats.pipes > 0:
            line3.append(f"{self.formatBigNumbers(stats.pipes)} Pipes")
        text = ["\n".join([line for line in (
            ", ".join(line1), ", ".join(line2), ", ".join(line3)) if line])]
        if stats.errors:
            text.append(
                f"{self.formatBigNumbers(len(stats.errors))} Failed to read")
        self.lblStats.config(text="\n".join(text))

    def updater(self):
        if self.scanner.busy():
            self.update()
            self.timerId = self.after(300, self.updater)
            return
        if self.timerId is not None:
            self.after_cancel(self.timerId)
            self.timerId = None
        self.update()
        self.lblRootDir.config(
            text=f"Finished analyzing after {time.time() - self.startTime:.2f}s", background="lawn green")


def ShowMessageBox(cbFunc, message):
    root = Tk()  # We need a main window
    root.attributes('-alpha', 0.0)
    root.wm_state('iconic')
    if not sys.platform.lower().startswith("win"):
        root.withdraw()
    cbFunc("Fast Properties", message)
    root.withdraw()


if __name__ == "__main__":
    if len(sys.argv) < 2:
        ShowMessageBox(messagebox.showinfo,
                       "Usage: fastproperties <Directory>")
        sys.exit(1)
    pathName = sys.argv[1]
    if os.name == 'posix':
        pathName = os.path.expanduser(pathName)
    if not os.path.exists(pathName):
        ShowMessageBox(messagebox.showerror, "Invalid directory!")
        sys.exit(1)
    app = FastProperties(pathName)
    app.mainloop()
