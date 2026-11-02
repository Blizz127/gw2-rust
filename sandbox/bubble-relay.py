"""Stand-in for the Banshee launcher's side of its GW2 bubble, for sandbox runs only.

    python3 -I bubble-relay.py BRIDGE_JSON UPSTREAM_HOST

Listens on each bridge listener's Unix socket and forwards it to UPSTREAM_HOST on
the Banshee service port for that role (cdn 6115, cligate 6114, auth 6113, game
6112), exactly as the launcher relays to the deployed server. The relay writes
<socket dir>/relay-ready once every socket listens.
"""
import asyncio, json, os, sys

PORTS = {"cdn": 6115, "cligate": 6114, "auth": 6113, "game": 6112}


async def pump(reader, writer):
    try:
        while data := await reader.read(65536):
            writer.write(data)
            await writer.drain()
    except OSError:
        pass
    finally:
        try:
            writer.close()
        except Exception:
            pass


async def main():
    config = json.load(open(sys.argv[1]))
    host = sys.argv[2]
    servers = []
    for listener in config["listeners"]:
        port = PORTS[listener["name"]]

        async def handle(reader, writer, port=port):
            try:
                up_reader, up_writer = await asyncio.open_connection(host, port)
            except OSError:
                writer.close()
                return
            await asyncio.gather(pump(reader, up_writer), pump(up_reader, writer))

        if os.path.exists(listener["socket"]):
            os.unlink(listener["socket"])
        servers.append(await asyncio.start_unix_server(handle, listener["socket"]))
    open(os.path.join(os.path.dirname(config["listeners"][0]["socket"]), "relay-ready"), "w").close()
    await asyncio.Event().wait()


asyncio.run(main())
