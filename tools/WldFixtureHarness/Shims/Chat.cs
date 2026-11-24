// S1-J-3 golden-fixture harness - the ChatHelper state the extracted
// announcement bodies read (Generated/ChatHelperSave.cs). DisplayMessage's
// UI writes go to Main.NewTextMultiline / Player.chatOverhead shims; the
// message cache is chat-only state, never serialized.

using System;
using System.Collections.Generic;
using Microsoft.Xna.Framework;

namespace Terraria.Chat
{
    public static partial class ChatHelper
    {
        // ChatHelper.cs:13 + the cctor :130-133. CacheMessage (:100-103) is
        // only reached when ShouldCacheMessage() - netMode 1 + gameMenu;
        // the fixture runs netMode 0, so the list stays empty.
        private static List<Tuple<string, Color>> _cachedMessages
            = new List<Tuple<string, Color>>();

        private static void CacheMessage(string message, Color color)
            => _cachedMessages.Add(new Tuple<string, Color>(message, color));

        // ChatHelper.cs:67-70 - the dedServ broadcast's login filter; the
        // fixture never reaches it (netMode 0 / dedServ false).
        public static bool OnlySendToPlayersWhoAreLoggedIn(int clientIndex)
            => Netplay.Clients[clientIndex].State == 10;
    }
}
