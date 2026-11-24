using System;
using Microsoft.Xna.Framework;

namespace ReLogic.OS.Base;

public abstract class SDLWindowService : IWindowService
{
	public float GetScaling()
	{
		return 1f;
	}

	public void SetQuickEditEnabled(bool enabled)
	{
	}

	public void SetUnicodeTitle(GameWindow window, string title)
	{
		window.Title = title;
	}

	public void StartFlashingIcon(GameWindow window)
	{
	}

	public void StopFlashingIcon(GameWindow window)
	{
	}

	public void Activate(GameWindow window)
	{
	}

	public bool IsSizeable(GameWindow window)
	{
		throw new PlatformNotSupportedException();
	}

	public void SetPosition(GameWindow window, int x, int y)
	{
	}

	public Rectangle GetBounds(GameWindow window)
	{
		throw new PlatformNotSupportedException();
	}
}
