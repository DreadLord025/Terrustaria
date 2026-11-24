using System;
using System.Collections.Generic;
using System.IO;

namespace ReLogic.Content.Sources;

public class FileSystemContentSource : IContentSource
{
	private readonly string _basePath;

	private readonly Dictionary<string, string> _nameToAbsolutePath = new Dictionary<string, string>();

	private readonly RejectedAssetCollection _rejections = new RejectedAssetCollection();

	public IContentValidator ContentValidator { get; set; }

	public int FileCount => _nameToAbsolutePath.Count;

	public string FileWatcherPath => _basePath;

	public FileSystemContentSource(string basePath)
	{
		_basePath = Path.GetFullPath(basePath);
		if (!_basePath.EndsWith("/") && !_basePath.EndsWith("\\"))
		{
			_basePath += Path.DirectorySeparatorChar;
		}
		Refresh();
	}

	public bool HasAsset(string assetName)
	{
		if (_rejections.IsRejected(assetName))
		{
			return false;
		}
		return _nameToAbsolutePath.ContainsKey(assetName);
	}

	public List<string> GetAllAssetsStartingWith(string assetNameStart)
	{
		List<string> list = new List<string>();
		foreach (string key in _nameToAbsolutePath.Keys)
		{
			if (key.ToLower().StartsWith(assetNameStart))
			{
				list.Add(key);
			}
		}
		return list;
	}

	public string GetExtension(string assetName)
	{
		if (!_nameToAbsolutePath.TryGetValue(assetName, out var value))
		{
			throw AssetLoadException.FromMissingAsset(assetName);
		}
		return Path.GetExtension(value) ?? "";
	}

	public Stream OpenStream(string assetName)
	{
		if (!_nameToAbsolutePath.TryGetValue(assetName, out var value))
		{
			throw AssetLoadException.FromMissingAsset(assetName);
		}
		if (!File.Exists(value))
		{
			throw AssetLoadException.FromMissingAsset(assetName);
		}
		try
		{
			return File.OpenRead(value);
		}
		catch (Exception innerException)
		{
			throw AssetLoadException.FromMissingAsset(assetName, innerException);
		}
	}

	public void Refresh()
	{
		_nameToAbsolutePath.Clear();
		if (Directory.Exists(_basePath))
		{
			string[] files = Directory.GetFiles(_basePath, "*", SearchOption.AllDirectories);
			for (int i = 0; i < files.Length; i++)
			{
				string fullPath = Path.GetFullPath(files[i]);
				string text = Path.GetExtension(fullPath) ?? "";
				string path = fullPath.Substring(_basePath.Length, fullPath.Length - text.Length - _basePath.Length);
				path = AssetPathHelper.CleanPath(path);
				_nameToAbsolutePath[path] = fullPath;
			}
		}
	}

	public void RejectAsset(string assetName, IRejectionReason reason)
	{
		_rejections.Reject(assetName, reason);
	}

	public void ClearRejections()
	{
		_rejections.Clear();
	}

	public bool TryGetRejections(List<string> rejectionReasons)
	{
		return _rejections.TryGetRejections(rejectionReasons);
	}
}
