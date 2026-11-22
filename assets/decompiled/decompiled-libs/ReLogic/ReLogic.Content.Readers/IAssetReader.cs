using System.IO;

namespace ReLogic.Content.Readers;

public interface IAssetReader
{
	T FromStream<T>(Stream stream) where T : class;
}
