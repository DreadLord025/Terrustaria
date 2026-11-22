using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.RegularExpressions;
using Ionic.Zip;

namespace ReLogic.Content.Sources;

public class ZipContentSource : IContentSource, IDisposable
{
	private readonly ZipFile _zipFile;

	private readonly Dictionary<string, ZipEntry> _entries = new Dictionary<string, ZipEntry>();

	private readonly string _basePath;

	private bool _isDisposed;

	private readonly RejectedAssetCollection _rejections = new RejectedAssetCollection();

	public int EntryCount => _entries.Count;

	public IContentValidator ContentValidator { get; set; }

	public string FileWatcherPath => null;

	public ZipContentSource(string path)
		: this(path, "")
	{
	}

	public ZipContentSource(string path, string contentDir)
		: this(ZipFile.Read(path), contentDir)
	{
	}

	public ZipContentSource(ZipFile zip, string contentDir)
	{
		_zipFile = zip;
		if (ZipPathContainsInvalidCharacters(contentDir))
		{
			throw new ArgumentException("Content directory cannot contain \"..\"", "contentDir");
		}
		_basePath = CleanZipPath(contentDir);
		Refresh();
	}

	public bool HasAsset(string assetName)
	{
		if (!_entries.TryGetValue(assetName, out var _))
		{
			return false;
		}
		if (_rejections.IsRejected(assetName))
		{
			return false;
		}
		return true;
	}

	public List<string> GetAllAssetsStartingWith(string assetNameStart)
	{
		List<string> list = new List<string>();
		foreach (string key in _entries.Keys)
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
		if (!_entries.TryGetValue(assetName, out var value))
		{
			throw AssetLoadException.FromMissingAsset(assetName);
		}
		return Path.GetExtension(value.FileName) ?? "";
	}

	public Stream OpenStream(string assetName)
	{
		if (!_entries.TryGetValue(assetName, out var value))
		{
			throw AssetLoadException.FromMissingAsset(assetName);
		}
		MemoryStream memoryStream = new MemoryStream((int)value.UncompressedSize);
		value.Extract((Stream)memoryStream);
		memoryStream.Position = 0L;
		return memoryStream;
	}

	public void Refresh()
	{
		_entries.Clear();
		foreach (ZipEntry item in _zipFile.Entries.Where((ZipEntry entry) => !entry.IsDirectory && entry.FileName.StartsWith(_basePath)))
		{
			string fileName = item.FileName;
			string text = Path.GetExtension(fileName) ?? "";
			string path = fileName.Substring(_basePath.Length, fileName.Length - text.Length - _basePath.Length);
			path = AssetPathHelper.CleanPath(path);
			_entries[path] = item;
		}
	}

	private static bool ZipPathContainsInvalidCharacters(string path)
	{
		if (!path.Contains("../"))
		{
			return path.Contains("..\\");
		}
		return true;
	}

	private static string CleanZipPath(string path)
	{
		path = path.Replace('\\', '/');
		path = Regex.Replace(path, "^[./]+", "");
		if (path.Length != 0 && !path.EndsWith("/"))
		{
			path += "/";
		}
		return path;
	}

	protected virtual void Dispose(bool disposing)
	{
		if (!_isDisposed)
		{
			if (disposing)
			{
				_entries.Clear();
				_zipFile.Dispose();
			}
			_isDisposed = true;
		}
	}

	public void Dispose()
	{
		Dispose(disposing: true);
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
