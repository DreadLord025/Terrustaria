namespace ReLogic.Content;

public class ContentRejectionFromFailedLoad_ByAssetExtensionType : IRejectionReason
{
	public string GetReason()
	{
		return "Only textures of type '.png' and '.xnb' may be loaded.";
	}
}
