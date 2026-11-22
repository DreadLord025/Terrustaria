using Microsoft.Xna.Framework;

namespace ReLogic.OS;

public interface IWindowService
{
	void SetUnicodeTitle(GameWindow window, string title);

	void StartFlashingIcon(GameWindow window);

	void StopFlashingIcon(GameWindow window);

	float GetScaling();

	void SetQuickEditEnabled(bool enabled);

	void Activate(GameWindow window);

	bool IsSizeable(GameWindow window);

	void SetPosition(GameWindow window, int x, int y);

	Rectangle GetBounds(GameWindow window);
}
