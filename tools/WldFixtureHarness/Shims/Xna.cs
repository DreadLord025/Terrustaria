// S1-J-3 golden-fixture harness — minimal XNA math types.
//
// The save path only ever touches Vector2.X/Y (SaveNPCs / WriteVector2),
// Color.R/G/B (Player.Serialize's seven colours) and Point.X/Y (the pressure
// plates, the town rooms, the chest anchor pre-pass). These are the FNA/XNA
// value types reduced to exactly the members the extracted bodies read; no
// arithmetic on them is on the save path, so nothing else is modelled.

namespace Microsoft.Xna.Framework
{
    public struct Vector2
    {
        public float X;
        public float Y;
        public Vector2(float x, float y) { X = x; Y = y; }
        public static Vector2 Zero => new Vector2(0f, 0f);
    }

    public struct Point
    {
        public int X;
        public int Y;
        public Point(int x, int y) { X = x; Y = y; }
        public static bool operator ==(Point a, Point b) => a.X == b.X && a.Y == b.Y;
        public static bool operator !=(Point a, Point b) => !(a == b);
        public override bool Equals(object obj) => obj is Point p && this == p;
        public override int GetHashCode() => (X << 16) ^ Y;
    }

    public struct Color
    {
        public byte R;
        public byte G;
        public byte B;
        public byte A;
        public Color(byte r, byte g, byte b) { R = r; G = g; B = b; A = 255; }
        public Color(byte r, byte g, byte b, byte a) { R = r; G = g; B = b; A = a; }
    }

    public struct Vector3
    {
        public float X;
        public float Y;
        public float Z;
        public static Vector3 operator *(Vector3 v, float s) => new Vector3 { X = v.X * s, Y = v.Y * s, Z = v.Z * s };
    }

    // FNA's MathHelper.Lerp — `value1 + (value2 - value1) * amount`
    // (the only member Utils.Remap reaches).
    public static class MathHelper
    {
        public static float Lerp(float value1, float value2, float amount)
            => value1 + (value2 - value1) * amount;
    }
}
