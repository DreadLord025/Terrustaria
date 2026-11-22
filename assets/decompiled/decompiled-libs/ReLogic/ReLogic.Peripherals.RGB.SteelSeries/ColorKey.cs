using Microsoft.Xna.Framework;
using Newtonsoft.Json.Linq;

namespace ReLogic.Peripherals.RGB.SteelSeries;

internal class ColorKey
{
	private const int TimesToDenyIdenticalColors = 30;

	public string EventName;

	public string TriggerName;

	private Color _colorToShow;

	private bool _needsToSendMessage;

	public bool IsVisible;

	private int _timesDeniedColorRepeats;

	public void UpdateColor(Color color, bool isVisible)
	{
		IsVisible = isVisible;
		if (_colorToShow == color && _timesDeniedColorRepeats < 30)
		{
			_timesDeniedColorRepeats++;
			return;
		}
		_timesDeniedColorRepeats = 0;
		_colorToShow = color;
		_needsToSendMessage = true;
	}

	public JObject TryGettingRequest()
	{
		//IL_0011: Unknown result type (might be due to invalid IL or missing references)
		//IL_0017: Expected Obj, but got Unknown
		//IL_0068: Unknown result type (might be due to invalid IL or missing references)
		//IL_006e: Expected Obj, but got Unknown
		//IL_007b: Unknown result type (might be due to invalid IL or missing references)
		//IL_0081: Expected Obj, but got Unknown
		//IL_008d: Unknown result type (might be due to invalid IL or missing references)
		//IL_0092: Unknown result type (might be due to invalid IL or missing references)
		//IL_00a8: Unknown result type (might be due to invalid IL or missing references)
		//IL_00b5: Expected Obj, but got Unknown
		if (!_needsToSendMessage)
		{
			return null;
		}
		_needsToSendMessage = false;
		JObject val = new JObject();
		val.Add("red", JToken.op_Implicit(_colorToShow.R));
		val.Add("green", JToken.op_Implicit(_colorToShow.G));
		val.Add("blue", JToken.op_Implicit(_colorToShow.B));
		JObject val2 = new JObject();
		val2.Add(TriggerName, (JToken)(object)val);
		JObject val3 = new JObject();
		val3.Add("frame", (JToken)(object)val2);
		JObject val4 = new JObject();
		val4.Add("event", JToken.op_Implicit(EventName));
		val4.Add("data", (JToken)(object)val3);
		return val4;
	}
}
