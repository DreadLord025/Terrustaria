using System;
using System.IO;
using ReLogic.Content.Sources;

namespace ReLogic.Content;

public interface IAssetLoader
{
	bool TryLoad<T>(string assetName, IContentSource source, out T resultAsset) where T : class;

	bool TryLoad<T>(string extension, Func<Stream> getStream, out T resultAsset) where T : class;
}
