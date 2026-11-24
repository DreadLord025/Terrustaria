using System.Collections;
using System.Collections.Generic;
using Microsoft.Xna.Framework.Input;

namespace ReLogic.Peripherals.RGB;

internal class HotkeyCollection : IEnumerable<RgbKey>, IEnumerable
{
	private Dictionary<Keys, RgbKey> _keys = new Dictionary<Keys, RgbKey>();

	public RgbKey BindKey(Keys key, string keyTriggerName)
	{
		if (!_keys.ContainsKey(key))
		{
			_keys.Add(key, new RgbKey(key, keyTriggerName));
		}
		return _keys[key];
	}

	public void UnbindKey(Keys key)
	{
		_keys.Remove(key);
	}

	public IEnumerator<RgbKey> GetEnumerator()
	{
		return _keys.Values.GetEnumerator();
	}

	IEnumerator IEnumerable.GetEnumerator()
	{
		return _keys.Values.GetEnumerator();
	}

	public void UpdateAll(float timeElapsed)
	{
		foreach (RgbKey value in _keys.Values)
		{
			value.Update(timeElapsed);
		}
	}
}
