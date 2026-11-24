using System;
using ReLogic.Localization.IME;

namespace ReLogic.OS.Windows;

internal class WindowsPlatform : Platform
{
	private WindowsMessageHook _wndProcHook;

	private bool _disposedValue;

	public WindowsPlatform()
		: base(PlatformType.Windows)
	{
		RegisterService((IClipboard)new Clipboard());
		RegisterService((IPathService)new PathService());
		RegisterService((IWindowService)new WindowService());
		RegisterService((IImeService)new UnsupportedPlatformIme());
	}

	public override void InitializeClientServices(IntPtr windowHandle)
	{
		if (_wndProcHook == null)
		{
			_wndProcHook = new WindowsMessageHook(windowHandle);
		}
		RegisterService((IImeService)new WindowsIme(_wndProcHook, windowHandle));
		RegisterService((IMouseNotifier)new WindowsMouseNotifier(_wndProcHook));
	}

	protected override void Dispose(bool disposing)
	{
		if (!_disposedValue)
		{
			if (disposing && _wndProcHook != null)
			{
				_wndProcHook.Dispose();
				_wndProcHook = null;
			}
			_disposedValue = true;
			base.Dispose(disposing);
		}
	}
}
