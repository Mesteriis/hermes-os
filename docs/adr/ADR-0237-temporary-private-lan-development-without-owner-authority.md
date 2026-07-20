# ADR-0237: Временный private-LAN development без owner authority

Статус: Принято  
Дата: 2026-07-20  
Состояние реализации: реализован. `--dangerous-lan-development` является
process-local флагом и монтирует только technical health/readiness surface.

Заменяет:

- [ADR-0235: Private-LAN developer mode](ADR-0235-private-lan-developer-mode.md).

Зависит от:

- [ADR-0205: Core Gateway и транспорт клиентских приложений](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0216: Private Kernel Control Store на SQLite](ADR-0216-private-kernel-control-store-with-sqlite.md);
- [ADR-0218: Owner/device identity, enrollment и offline recovery](ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md);
- [ADR-0232: Browser client identity and same-origin Gateway session](ADR-0232-browser-client-device-identity-and-same-origin-session.md).

## Контекст

Durable LAN setting из ADR-0235 смешивал transport convenience с owner
authority: перезапуск сохранял unauthenticated owner access, а private network
не является proof владения device key. Это противоречит owner/device boundary
ADR-0218 и не может быть исправлено только warning в UI.

## Решение

`--dangerous-lan-development` разрешён только как явный process-local запуск
на literal private-LAN address без TLS. Он не записывается в Control Store,
settings, release artifact или backup и перестаёт действовать после process
exit.

В таком listener Gateway публикует только bounded technical routes:

- health;
- readiness;
- sanitized runtime status без owner, session, credential или content data.

Owner APIs, pairing, enrollment, session/bootstrap, SSE, Gateway client RPC и
все owner-specific routes не монтируются. Наличие уже enrolled owner не меняет
это правило. Получить owner authority можно только через ранее enrolled P-256
device proof в обычном authenticated transport profile.

Initial enrollment остаётся доступным только через inherited Tauri/platform
signer path или HTTPS secure context после explicit local CLI approval. LAN
listener не является enrollment transport.

TLS input и этот флаг взаимоисключающие; wildcard, loopback и public bind не
разрешаются. Diagnostics должны быть typed/redacted и не раскрывать причину
отказа, позволяющую получить private state.

## Проверка

Executable checks обязаны доказывать:

- migration 35→36 удаляет durable LAN/operator bypass state;
- флаг не меняет Control Store и не переживает restart;
- LAN listener не имеет owner, pairing, enrollment, client RPC или realtime
  routes;
- owner route в LAN profile получает typed unauthenticated/unavailable
  response без sensitive diagnostics;
- normal paired profile всё ещё требует enrolled device proof.

## Последствия

LAN режим годится для network/probe debugging, но не для разработки
authenticated client flow. Такой flow запускается через обычный paired local
profile или test harness с real device proof; отдельная future development
topology потребует нового ADR и executable abuse evidence.
