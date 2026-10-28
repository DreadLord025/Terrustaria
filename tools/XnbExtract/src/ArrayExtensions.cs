namespace TConvert.Extract;

/// <summary>Extension helpers used by the imported LZX decoder sources.</summary>
internal static class ArrayExtensions
{
    public static void Fill<T>(this T[] array, T value) => System.Array.Fill(array, value);
}
