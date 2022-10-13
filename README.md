lila-engine
===========

Broker for communication between external engine providers and clients.
See https://github.com/lichess-org/external-engine for providers.

Provides:

* [`https://engine.lichess.ovh/api/external-engine/{id}/analyse`](https://lichess.org/api#tag/External-engine-(draft)/operation/apiExternalEngineAnalyse)
* [`https://engine.lichess.ovh/api/external-engine/work`](https://lichess.org/api#tag/External-engine-(draft)/operation/apiExternalEngineAcquire)
* [`https://engine.lichess.ovh/api/external-engine/work/{id}`](https://lichess.org/api#tag/External-engine-(draft)/operation/apiExternalEngineSubmit)

Usage
-----

```
LILA_ENGINE_LOG=lila_engine=debug,tower_http=debug cargo run
```

License
-------

Licensed under the GNU Affero General Public License v3. See the `LICENSE` file
for details.
