using System;
using ReLogic.OS.Base;

namespace ReLogic.OS.Linux;

internal class Clipboard : ReLogic.OS.Base.Clipboard
{
	protected override string GetClipboard()
	{
		throw new PlatformNotSupportedException();
	}

	protected override void SetClipboard(string text)
	{
	}
}
