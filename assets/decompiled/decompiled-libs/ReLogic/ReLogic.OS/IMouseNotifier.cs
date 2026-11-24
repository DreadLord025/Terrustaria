using System;

namespace ReLogic.OS;

public interface IMouseNotifier
{
	void ForceCursorHidden();

	void AddMouseHandler(Action<bool> action);

	void RemoveMouseHandler(Action<bool> action);
}
