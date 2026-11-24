using System;
using System.IO;
using ReLogic.Content.Sources;

namespace ReLogic.Content;

public class AssetLoader : IAssetLoader
{
	private readonly AssetReaderCollection _readers;

	public AssetLoader(AssetReaderCollection readers)
	{
		_readers = readers;
	}

	public bool TryLoad<T>(string assetName, IContentSource source, out T resultAsset) where T : class
	{
		return TryLoad<T>(source.GetExtension(assetName), () => source.OpenStream(assetName), out resultAsset);
	}

	public bool TryLoad<T>(string extension, Func<Stream> getStream, out T resultAsset) where T : class
	{
		resultAsset = null;
		if (!_readers.CanReadExtension(extension))
		{
			return false;
		}
		using (Stream stream = getStream())
		{
			resultAsset = _readers.Read<T>(stream, extension);
		}
		return true;
	}
}
