lila-engine
===========

Broker for communication between external engine providers and clients.

Endpoints:

* [`https://engine.lichess.ovh/api/external-engine/{id}/analyse`](https://lichess.org/api#tag/External-engine/operation/apiExternalEngineAnalyse)
* [`https://engine.lichess.ovh/api/external-engine/work`](https://lichess.org/api#tag/External-engine/operation/apiExternalEngineAcquire)
* [`https://engine.lichess.ovh/api/external-engine/work/{id}`](https://lichess.org/api#tag/External-engine/operation/apiExternalEngineSubmit)

Providers
---------

See https://github.com/lichess-org/external-engine for external engine
providers.

Usage
-----

```
LILA_ENGINE_LOG=lila_engine=debug,tower_http=debug cargo run -- --bind 127.0.0.1:9663
```

License
-------

Licensed under the GNU Affero General Public License v3. See the `LICENSE` file
for details.
