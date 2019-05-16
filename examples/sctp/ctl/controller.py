#!/usr/bin/env python3

import asyncio
import struct
async def connect_and_test(inter_message_gap):
    (reader, writer) = await asyncio.open_connection('127.0.0.1', 8001)
    print("Connected")
    times = int(300.0 / float(inter_message_gap))
    for i in range(0, times):
        to_write = struct.pack('qBBBBBB', 2,  0x68, 0x05, 0xca, 0x33, 0xfd, 0xc9)
        writer.write(to_write)
        await writer.drain()
        await asyncio.sleep(inter_message_gap)

if __name__ == "__main__":
    loop = asyncio.get_event_loop()
    loop.run_until_complete(connect_and_test(0.0001))
    loop.close()
