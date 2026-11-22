using System.Collections.Generic;
using System.IO;
using ReLogic.Content.Readers;

namespace ReLogic.Content;

public class AssetReaderCollection
{
	private readonly Dictionary<string, IAssetReader> _readersByExtension = new Dictionary<string, IAssetReader>();

	public void RegisterReader(IAssetReader reader, params string[] extensions)
	{
		foreach (string text in extensions)
		{
			_readersByExtension[text.ToLower()] = reader;
		}
	}

	public T Read<T>(Stream stream, string extension) where T : class
	{
		if (!_readersByExtension.TryGetValue(extension.ToLower(), out var value))
		{
			throw AssetLoadException.FromMissingReader(extension);
		}
		return value.FromStream<T>(stream);
	}

	public bool CanReadExtension(string extension)
	{
		return _readersByExtension.ContainsKey(extension.ToLower());
	}
}
