import * as broadcastChannel from "ext:deno_broadcast_channel/01_broadcast_channel.js";
import { core } from "ext:core/mod.js";

function broadcast_serialize(data) {
    let uint8Array = core.serialize(data);
    return Array.from(uint8Array);
}

import { applyToGlobal, nonEnumerable } from 'ext:rustyscript/rustyscript.js';
applyToGlobal({
    BroadcastChannel: nonEnumerable(broadcastChannel.BroadcastChannel),
    broadcast_serialize: nonEnumerable(broadcast_serialize),
    broadcast_deserialize: nonEnumerable(core.deserialize),
});
