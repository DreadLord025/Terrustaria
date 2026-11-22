using System;
using System.Drawing;
using System.Runtime.InteropServices;
using System.Windows.Forms;
using Microsoft.Xna.Framework;

namespace ReLogic.OS.Windows;

internal class WindowService : IWindowService
{
	public float GetScaling()
	{
		try
		{
			IntPtr hdc = System.Drawing.Graphics.FromHwnd(IntPtr.Zero).GetHdc();
			int deviceCaps = NativeMethods.GetDeviceCaps(hdc, NativeMethods.DeviceCap.VertRes);
			return (float)NativeMethods.GetDeviceCaps(hdc, NativeMethods.DeviceCap.DesktopVertRes) / (float)deviceCaps;
		}
		catch (Exception)
		{
			return 1f;
		}
	}

	public void SetQuickEditEnabled(bool enabled)
	{
		IntPtr stdHandle = NativeMethods.GetStdHandle(NativeMethods.StdHandleType.Input);
		if (NativeMethods.GetConsoleMode(stdHandle, out var lpMode))
		{
			lpMode = ((!enabled) ? (lpMode & ~NativeMethods.ConsoleMode.QuickEditMode) : (lpMode | NativeMethods.ConsoleMode.QuickEditMode));
			NativeMethods.SetConsoleMode(stdHandle, lpMode);
		}
	}

	public void SetUnicodeTitle(GameWindow window, string title)
	{
		NativeMethods.WndProcCallback wndProcCallback = NativeMethods.DefWindowProc;
		int dwNewLong = NativeMethods.SetWindowLong(window.Handle, -4, (int)Marshal.GetFunctionPointerForDelegate((Delegate)wndProcCallback));
		window.Title = title;
		NativeMethods.SetWindowLong(window.Handle, -4, dwNewLong);
		GC.KeepAlive(wndProcCallback);
	}

	public void StartFlashingIcon(GameWindow window)
	{
		NativeMethods.FlashInfo flashInfo = NativeMethods.FlashInfo.CreateStart(window.Handle);
		NativeMethods.FlashWindowEx(ref flashInfo);
	}

	public void StopFlashingIcon(GameWindow window)
	{
		NativeMethods.FlashInfo flashInfo = NativeMethods.FlashInfo.CreateStop(window.Handle);
		NativeMethods.FlashWindowEx(ref flashInfo);
	}

	public void Activate(GameWindow window)
	{
		((Form)Control.FromHandle(window.Handle)).Activate();
	}

	public bool IsSizeable(GameWindow window)
	{
		Form form = (Form)Control.FromHandle(window.Handle);
		if (form.WindowState == FormWindowState.Normal)
		{
			return form.FormBorderStyle == FormBorderStyle.Sizable;
		}
		return false;
	}

	public void SetPosition(GameWindow window, int x, int y)
	{
		((Form)Control.FromHandle(window.Handle)).Location = new System.Drawing.Point(x, y);
	}

	public Microsoft.Xna.Framework.Rectangle GetBounds(GameWindow window)
	{
		Form form = (Form)Control.FromHandle(window.Handle);
		return new Microsoft.Xna.Framework.Rectangle(form.Bounds.X, form.Bounds.Y, form.Bounds.Width, form.Bounds.Height);
	}
}
