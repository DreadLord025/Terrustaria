// Terraria XNB -> PNG extractor.
// XNB container v5 with Xbox XMemCompress (LZX, 16-bit window, headered blocks).
// LZX decoder: MonoGame/liblzx port (see src/LzxDecoder.cs).
using System.Buffers.Binary;
using System.Collections.Concurrent;
using System.IO.Compression;
using System.Text;

namespace TConvert.Extract;

internal static class Program
{
    private const int HeaderSize = 14;

    private static int Main(string[] args)
    {
        if (args.Length < 2)
        {
            Console.Error.WriteLine("usage: XnbExtract <contentDir> <outDir> [nameFilter]");
            return 1;
        }

        string inDir = Path.GetFullPath(args[0]);
        string outDir = Path.GetFullPath(args[1]);
        string filter = args.Length > 2 ? args[2] : null;

        Directory.CreateDirectory(outDir);

        var files = Directory.EnumerateFiles(inDir, "*.xnb", SearchOption.AllDirectories)
            .Where(f => filter == null || Path.GetFileName(f).Contains(filter, StringComparison.OrdinalIgnoreCase))
            .ToArray();

        Console.WriteLine($"extracting {files.Length} xnb files -> {outDir}");

        int ok = 0, skipped = 0, failed = 0;
        var errors = new ConcurrentBag<string>();
        var sw = System.Diagnostics.Stopwatch.StartNew();

        Parallel.ForEach(files, new ParallelOptions { MaxDegreeOfParallelism = Environment.ProcessorCount },
            file =>
            {
                string rel = Path.GetRelativePath(inDir, file);
                string dst = Path.Combine(outDir, Path.ChangeExtension(rel, ".png"));
                try
                {
                    if (Extract(file, dst)) Interlocked.Increment(ref ok);
                    else Interlocked.Increment(ref skipped);
                }
                catch (Exception e)
                {
                    Interlocked.Increment(ref failed);
                    errors.Add($"{rel}: {e.GetType().Name}: {e.Message}");
                }
            });

        Console.WriteLine($"ok={ok} skipped={skipped} failed={failed} in {sw.Elapsed.TotalSeconds:F1}s");
        foreach (var e in errors.Take(20)) Console.WriteLine("  " + e);
        return failed == 0 ? 0 : 2;
    }

    /// <summary>Returns true if an image was written, false if the asset is not a Texture2D.</summary>
    private static bool Extract(string inputFile, string outputFile)
    {
        byte[] raw = File.ReadAllBytes(inputFile);
        using var ms = new MemoryStream(raw, writable: false);
        using var reader = new BinaryReader(ms);

        if (Encoding.ASCII.GetString(reader.ReadBytes(3)) != "XNB")
            throw new InvalidDataException("not an XNB file");

        reader.ReadByte(); // target platform
        int version = reader.ReadByte();
        if (version != 5) throw new InvalidDataException($"unsupported XNB version {version}");

        bool compressed = (reader.ReadByte() & 0x80) != 0;
        int compressedSize = reader.ReadInt32();
        int decompressedSize = compressed ? reader.ReadInt32() : compressedSize;

        MemoryStream payload;
        if (compressed)
        {
            payload = new MemoryStream(decompressedSize);
            var lzx = new LzxDecoder();
            lzx.Decompress(reader, compressedSize - HeaderSize, payload, decompressedSize);
            payload.Position = 0;
        }
        else
        {
            payload = new MemoryStream(raw, HeaderSize, raw.Length - HeaderSize, writable: false);
        }

        using var br = new BinaryReader(payload);

        int readerCount = Read7BitEncodedInt(br);
        var readerNames = new List<string>(readerCount);
        for (int i = 0; i < readerCount; i++)
        {
            int len = Read7BitEncodedInt(br);
            readerNames.Add(Encoding.UTF8.GetString(br.ReadBytes(len)));
            br.ReadInt32(); // version
        }

        int sharedResources = Read7BitEncodedInt(br);
        if (sharedResources != 0)
            throw new InvalidDataException($"unexpected shared resources: {sharedResources}");

        int typeId = Read7BitEncodedInt(br);
        if (typeId == 0) return false;
        if (typeId < 1 || typeId > readerNames.Count)
            throw new InvalidDataException($"bad type id {typeId} (readers={readerNames.Count})");

        string assetReader = readerNames[typeId - 1];
        if (assetReader.Contains("SpriteFontReader", StringComparison.Ordinal)
            || assetReader.Contains("DynamicSpriteFontReader", StringComparison.Ordinal))
        {
            // ReLogic DynamicSpriteFont: layout not yet reverse-engineered (see
            // docs/ROADMAP.md "pixel-exact UI fonts"); skip instead of failing.
            return false;
        }
        if (!assetReader.Contains("Texture2DReader", StringComparison.Ordinal))
        {
            if (Environment.GetEnvironmentVariable("XNB_DEBUG") == "1")
                Console.WriteLine($"  not a texture ({assetReader}): {Path.GetFileName(inputFile)}");
            return false;
        }

        int surfaceFormat = br.ReadInt32(); // 0=Color(BGRA), 4=Dxt1, 5=Dxt3, 6=Dxt5
        int width = br.ReadInt32();
        int height = br.ReadInt32();
        int mipCount = br.ReadInt32();
        if (mipCount < 1) throw new InvalidDataException($"bad mip count {mipCount}");

        if (Environment.GetEnvironmentVariable("XNB_INFO") == "1")

            Console.WriteLine($"{Path.GetFileName(inputFile)}: format={surfaceFormat} {width}x{height} mips={mipCount}$");


        byte[] pixels = null;
        for (int mip = 0; mip < mipCount; mip++)
        {
            int size = br.ReadInt32();
            byte[] data = br.ReadBytes(size);
            if (mip == 0)
            {
                pixels = surfaceFormat switch
                {
                    0 => data,
                    4 => DxtUtil.DecompressDxt1(data, width, height),
                    5 => DxtUtil.DecompressDxt3(data, width, height),
                    6 => DxtUtil.DecompressDxt5(data, width, height),
                    _ => throw new InvalidDataException($"unsupported surface format {surfaceFormat}")
                };
            }
        }

        if (pixels == null || pixels.Length < width * height * 4)
            throw new InvalidDataException("short pixel buffer");

        Directory.CreateDirectory(Path.GetDirectoryName(outputFile)!);
        using var fs = File.Create(outputFile);
        Png.WriteRgba(fs, width, height, pixels);
        return true;
    }

    private static int Read7BitEncodedInt(BinaryReader r)
    {
        int result = 0, shift = 0;
        while (true)
        {
            byte b = r.ReadByte();
            result |= (b & 0x7F) << shift;
            if ((b & 0x80) == 0) return result;
            shift += 7;
        }
    }

    /// <summary>Minimal dependency-free PNG writer (8-bit RGBA, no filtering).</summary>
    private static class Png
    {
        public static void WriteRgba(Stream s, int width, int height, byte[] rgba)
        {
            s.Write(new byte[] { 0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A });

            Span<byte> ihdr = stackalloc byte[13];
            BinaryPrimitives.WriteInt32BigEndian(ihdr[0..], width);
            BinaryPrimitives.WriteInt32BigEndian(ihdr[4..], height);
            ihdr[8] = 8;  // bit depth
            ihdr[9] = 6;  // color type RGBA
            ihdr[10] = 0; // compression
            ihdr[11] = 0; // filter
            ihdr[12] = 0; // interlace
            Chunk(s, "IHDR", ihdr.ToArray());

            int stride = width * 4;
            byte[] filtered = new byte[(stride + 1) * height];
            for (int y = 0; y < height; y++)
            {
                filtered[y * (stride + 1)] = 0;
                Buffer.BlockCopy(rgba, y * stride, filtered, y * (stride + 1) + 1, stride);
            }

            using var ms = new MemoryStream();
            using (var z = new ZLibStream(ms, CompressionLevel.Fastest, leaveOpen: true))
                z.Write(filtered, 0, filtered.Length);
            Chunk(s, "IDAT", ms.ToArray());
            Chunk(s, "IEND", Array.Empty<byte>());
        }

        private static void Chunk(Stream s, string type, byte[] data)
        {
            Span<byte> len = stackalloc byte[4];
            BinaryPrimitives.WriteInt32BigEndian(len, data.Length);
            s.Write(len);
            byte[] typeBytes = Encoding.ASCII.GetBytes(type);
            s.Write(typeBytes);
            s.Write(data);
            uint crc = Crc32(typeBytes, data);
            Span<byte> c = stackalloc byte[4];
            BinaryPrimitives.WriteUInt32BigEndian(c, crc);
            s.Write(c);
        }

        private static readonly uint[] Table = BuildTable();

        private static uint[] BuildTable()
        {
            var t = new uint[256];
            for (uint i = 0; i < 256; i++)
            {
                uint c = i;
                for (int k = 0; k < 8; k++) c = (c & 1) != 0 ? 0xEDB88320u ^ (c >> 1) : c >> 1;
                t[i] = c;
            }
            return t;
        }

        private static uint Crc32(byte[] a, byte[] b)
        {
            uint c = 0xFFFFFFFFu;
            foreach (byte x in a) c = Table[(c ^ x) & 0xFF] ^ (c >> 8);
            foreach (byte x in b) c = Table[(c ^ x) & 0xFF] ^ (c >> 8);
            return c ^ 0xFFFFFFFFu;
        }
    }
}
