# AstroHealth — dashboard zdrowia załogi stacji kosmicznej

Full-stackowy dashboard, który przystępnie prezentuje parametry życiowe astronautów bez
przeładowywania ich informacjami. Na górze panelu astronauty wyświetlają się alerty na całą
szerokość. Parametry są pogrupowane w zakładki w **bocznym pasku nawigacji**:
**Środowisko zewnętrzne**, **Parametry fizjologiczne** i **Samopoczucie**.

* **Backend:** Rust, architektura heksagonalna (porty i adaptery), workspace z osobnymi crate'ami.
* **Frontend:** React + TypeScript, Mantine.

**Główny cel architektoniczny to modularność.** Parametr zdrowotny jest *daną* (deklaratywną
definicją), a nie kodem rozrzuconym po warstwach. Z definicji wynikają: walidacja, statusy,
alerty, partycja w bazie, zakładka i widget. Szczegóły w sekcjach
[Jak dodać nowy parametr](#jak-dodać-nowy-parametr-zdrowotny) i [Jak dodać zakładkę](#jak-dodać-zakładkę).

| Profil astronauty: pasek boczny, alerty i zakładka domyślna | Zakładka „Samopoczucie” |
|---|---|
| ![profil](docs/profile.png) | ![samopoczucie](docs/wellbeing.png) |

| Zakładka „Środowisko zewnętrzne” |
|---|
| ![środowisko](docs/environment.png) |

![załoga](docs/crew.png)

---

## Szybki start

```bash
docker compose up --build
```

| Usługa | Adres | Opis |
|---|---|---|
| frontend | http://localhost:3000 | nginx serwuje SPA i przekazuje `/api` do backendu |
| backend | http://localhost:8080/api | REST API (axum) |
| db | localhost:5432 | PostgreSQL 16 (`astro` / `astro`, baza `astro_health`) |

Usługa `seed` uruchamia się przed backendem. Wykonuje migracje, tworzy partycje parametrów
i generuje dane demonstracyjne. W compose działa z flagą `--reset`, więc **przy każdym starcie
dane demo są generowane od nowa** i kończą się na „teraz”. Inaczej po kilku dniach reguła
„zaległy pomiar” oznaczyłaby wszystko jako zaległe. Aby zachować dane między restartami,
usuń `--reset` z `docker-compose.yml`.

### Konta demonstracyjne (hasło dla wszystkich: `orbita2026`)

| Login | Rola | Stacja | Scenariusz |
|---|---|---|---|
| `lekarz.aurora` | nadzorujący | Aurora | widzi 4 astronautów stacji Aurora, ich alerty i zakładki |
| `lekarz.selene` | nadzorujący | Selene | widzi 1 astronautę stacji Selene |
| `anna.kowalska` | astronautka | Aurora | **krytyczny** zanik mięśni łydki, ostrzeżenie dla kości i środowiska; 4 alerty, więc baner pokazuje „+1 kolejny” |
| `marcus.reyes` | astronauta | Aurora | ciśnienie, temperatura, kilka słabszych nocy; 7 alertów |
| `yuki.tanaka` | astronautka | Aurora | **krytyczny** wynik badania moczu (wapń), niska witamina D |
| `lena.hoffmann` | astronautka | Aurora | tylko alerty środowiskowe stacji (mniej niż 3) |
| `omar.haddad` | astronauta | Selene | **brak alertów**: zielony pasek „Wszystkie parametry w normie” |

Na stacji Aurora trwa burza słoneczna (ostrzeżenie o promieniowaniu), a wymazy z powierzchni
są zaległe od 20 dni (alert „warte uwagi”). Te alerty widzi każdy astronauta tej stacji.
Postacie i stacje są fikcyjne.

### Uruchomienie deweloperskie

```bash
docker compose up -d db
cd backend
export DATABASE_URL=postgres://astro:astro@localhost:5432/astro_health JWT_SECRET=dev
cargo run --bin seed -- --reset   # migracje + dane demo
cargo run --bin server            # API na :8080
cargo test                        # domena, przypadki użycia, JWT

cd ../frontend
npm install
npm run dev                       # http://localhost:5173 (proxy /api → :8080)
```

Zmienne backendu: `DATABASE_URL` (wymagana), `JWT_SECRET`, `JWT_TTL_MINUTES` (480),
`BIND_ADDR` (`0.0.0.0:8080`), `CORS_ORIGINS` (`http://localhost:5173`), `RUST_LOG`.

---

## Co widzi użytkownik

### Alerty na górze panelu

* Alerty to **kompaktowe banery na całą szerokość treści**, nad treścią każdej zakładki
  parametrów. Cała sekcja zajmuje mniej niż jedną trzecią wysokości ekranu, a każdy baner
  pokazuje ważność (ikona + etykieta), treść i czas pomiaru.
* Na zakładce **Alerty** banerów u góry nie ma, bo ta zakładka i tak pokazuje pełną listę.
* Alerty dotyczą parametrów spoza normy i są **posortowane od krytycznych, przez ostrzeżenia,
  do wartych uwagi** (np. zaległy pomiar).
* Każdy alert ma **ikonę ważności**: ośmiokąt dla krytycznego, trójkąt dla ostrzeżenia,
  „i” dla wartego uwagi. Ikony różnią się kształtem, więc ważność nie jest sygnalizowana
  wyłącznie kolorem.
* Każdy alert **linkuje do pełnego raportu**: przełącza na właściwą zakładkę, przewija do
  wykresu, tabeli badań lub mapy ciała i obrysowuje je. Na mapie ciała zaznacza też
  konkretny mięsień lub kość, a w tabeli badań podświetla wynik.
* Gdy nie ma alertów krytycznych ani ostrzeżeń, wyświetla się **zielony pasek** „Wszystkie
  parametry w normie”.
* Baner pokazuje **2 najważniejsze alerty** (`BANNER_LIMIT` w `AlertsBanner.tsx`). Pod nimi jest wyraźny element „+N kolejnych
  alertów · zobacz wszystkie”, który otwiera zakładkę **Alerty** z pełną listą pogrupowaną
  według ważności, ze szczegółami każdego wyniku.
* Jeden alert przypada na jeden parametr: wymienia najgorszą składową, a pozostałe streszcza
  („oraz 4 inne wyniki poza normą”). Szczegóły są w zakładce Alerty. Dzięki temu np. 6 mięśni
  nie generuje 6 alertów.

### Zakładki w bocznym pasku

Zakładki są w pasku po lewej stronie (`ProfileSidebar`). Nazwisko astronauty u góry paska jest
przyciskiem: najechanie, fokus z klawiatury lub stuknięcie pokazuje tooltip z danymi profilu
(dawniej karta profilowa na każdej zakładce). Treść tooltipa jest wspólna z widokiem nadzoru
(`AstronautProfileTooltip`). Przy każdej jest najgorszy status
jej parametrów, a przy „Alertach” liczba alertów. Na telefonie pasek chowa się pod ikoną menu.
Pasek jest widoczny tylko w profilu astronauty (także w karcie otwartej przez nadzór), nie na
liście załogi.

| Zakładka | Parametry | Widget |
|---|---|---|
| **Środowisko zewnętrzne** | jakość powietrza (mikroby szkodliwe, CFU/m³), próbki z powierzchni stacji (CFU/100 cm², harmonogram pobrań), promieniowanie (mSv/doba), natężenie pola grawitacyjnego (µg) | wykresy z trendem; próbki: wyniki ostatniego pobrania, termin następnego, tabela 5 ostatnich pobrań, historia |
| **Parametry fizjologiczne** *(domyślna)* | tętno, temperatura ciała, ciśnienie, zanik mięśni, zanik kości, badanie krwi, badanie moczu | wykresy; mięśnie i kości na sylwetce; badania jako tabele |
| **Samopoczucie** | stan psychiczny (notka), sen | łagodna notka; wykres punktacji snu i raport z ostatniej nocy |
| **Alerty** | wszystkie alerty | lista pogrupowana według ważności |

**Dlaczego domyślna jest zakładka „Parametry fizjologiczne”:** to rdzeń aplikacji, czyli
parametry życiowe samego astronauty. Pilne sprawy z pozostałych zakładek i tak są widoczne
w banerze alertów nad zakładkami, więc profil otwiera się na najistotniejszych danych bez
dokładania informacji. Środowisko jest wspólne dla stacji i zwykle stabilne. Samopoczucie
jest celowo delikatne i nie powinno być pierwszym, co widzi zestresowany użytkownik.
Wybór jest zapisany w jednym miejscu: `MetricGroup::default_tab()` w
`backend/crates/domain/src/metrics/catalog.rs`.

### Szczegóły widgetów

* **Zanik mięśni i zanik kości:** sylwetka (przód i tył), na której każdy mięsień lub kość
  jest osobnym, podświetlanym kształtem w kolorze swojego stanu. **Po najechaniu** na mięsień
  pojawia się tooltip z wykresem jego zaniku. Kliknięcie przypina wybrany mięsień w panelu
  obok, co działa też na ekranach dotykowych. Kości mają **inną paletę** niż mięśnie:
  błękit, fiolet i róż zamiast zieleni, pomarańczu i czerwieni. Legenda jest pod sylwetką.
* **Badania krwi i moczu:** wyniki ostatniego badania ze **statusem każdego wyniku**, wybór
  **poprzednich badań** z listy, kolumna „Poprzednio” ze strzałką trendu. Statusy historycznych
  badań liczy backend.
* **Sen:** wykres **ogólnej punktacji** (suma punktów za czas snu, wydajność, sen głęboki,
  REM, czas zasypiania i wybudzenia; maksymalnie 100) oraz **raport z ostatniej nocy** ze
  wszystkimi parametrami. Alert powstaje tylko na podstawie punktacji ogólnej, żeby
  pojedyncze składowe nie zasypywały użytkownika.
* **Stan psychiczny:** łagodnie sformułowana notka **bez statusów, ocen i kolorów
  ostrzegawczych**. Nigdy nie generuje alertów.
* **Parametry stacji** są oznaczone etykietą „Cała stacja”.

### Konta nadzorujące

Karta „Załoga” pokazuje status każdego astronauty **na każdą zakładkę** (najgorszy parametr
w grupie) oraz **liczniki alertów** według ważności. Najechanie lub fokus na nazwisku pokazuje
tooltip z danymi profilu astronauty (rola, agencja, stacja, dzień misji, start, ostatni pomiar). Zastępuje to 12 kolumn, po jednej na
parametr, które przy rosnącej liczbie parametrów stawałyby się nieczytelne. Profil astronauty
otwiera się w zamykanej karcie, z tymi samymi alertami i zakładkami co w widoku astronauty.

---

## Wybory technologiczne

| Obszar | Wybór | Uzasadnienie |
|---|---|---|
| HTTP w Rust | **axum** (+ tokio, tower-http) | Najpowszechniejszy framework ekosystemu tokio. Ma ekstraktory typowane, a `FromRequestParts` daje czysty punkt na uwierzytelnianie. |
| Baza danych | **PostgreSQL 16** | Partycjonowanie deklaratywne (osobna partycja na agregat parametru), `JSONB` na składowe pomiaru i `date_bin` do agregacji szeregów. |
| Dostęp do DB | **sqlx** | Asynchroniczny driver SQL bez ORM, z wbudowanymi migracjami. Pełna kontrola nad zapytaniami czasowymi. |
| Auth | **jsonwebtoken** (HS256) + **argon2** | Standardowe crate'y do JWT i hashowania haseł (Argon2 to rekomendacja OWASP). |
| UI | **Mantine 8** | Kompletna biblioteka komponentów React z dobrym wsparciem TS i ciemnym motywem, z własnym pakietem wykresów. |
| Wykresy | **@mantine/charts** (na **Recharts 3**) | Wykresy spójne stylistycznie z UI, oparte na najpopularniejszej bibliotece wykresów dla Reacta. |
| Dane po stronie klienta | **TanStack Query** | Cache i deduplikacja zapytań. Każdy widget pobiera dane niezależnie. |
| Wizualizacja ciała | **2D SVG (przód/tył)** | Model 3D byłby nieproporcjonalnie kosztowny. Schemat 2D z kształtem na każdy mięsień lub kość jest czytelny i łatwo rozszerzalny. |

---

## Architektura

```
                ┌──────────────────────── driving adapters ─────────────────────────┐
                │  adapter-http (axum)        seed binary / przyszły gateway urządzeń │
                └──────────────┬───────────────────────────────┬────────────────────┘
                               │ ports::inbound                │ MeasurementIngestion
                ┌──────────────▼───────────────────────────────▼────────────────────┐
                │ application: Auth, Crew, Health, Alert, Wellbeing, Ingestion      │
                │              AccessPolicy (autoryzacja), SnapshotReader           │
                │ ┌───────────────────────────────────────────────────────────────┐ │
                │ │ domain: MetricDefinition / MetricGroup / MetricCatalog,       │ │
                │ │         MetricHistory, AlertEngine + AlertRule, body_map,     │ │
                │ │         WellbeingNote, Subject (astronauta | stacja)          │ │
                │ └───────────────────────────────────────────────────────────────┘ │
                └──────────────┬───────────────────────────────┬────────────────────┘
                               │ ports::outbound               │
                ┌──────────────▼──────────────┐  ┌─────────────▼────────────────────┐
                │ adapter-postgres (sqlx)     │  │ adapter-auth (JWT, Argon2)       │
                └─────────────────────────────┘  └──────────────────────────────────┘
                                 server = composition root (wiring, config)
```

Kierunek zależności **wymusza kompilator**, bo każda warstwa to osobny crate:

| Crate | Zależy od | Zawartość |
|---|---|---|
| `domain` | tylko `chrono`, `uuid`, `thiserror` | Definicje parametrów i zakładek, walidacja, klasyfikacja, **reguły alertów**, punktacja pochodna, mapa ciała |
| `application` | `domain` | Porty, przypadki użycia, polityka dostępu, testy z adapterami in-memory |
| `adapter-postgres` | `application`, `domain`, sqlx | Repozytoria i migracje |
| `adapter-auth` | `application`, `domain` | `TokenService` (JWT), `PasswordHasher` (Argon2) |
| `adapter-http` | `application`, `domain`, axum | Routing, DTO, mapowanie błędów. Brak logiki biznesowej |
| `server` | wszystko | `main` (API) i `bin/seed` |

### Model domeny

* **`MetricDefinition`**: kod, nazwa, `group` (zakładka), `scope` (`Astronaut` albo `Station`),
  typ prezentacji (`line`, `area`, `bar`, `table`, `body`), interwał pomiarów, `alert_policy`
  oraz składowe. Każda składowa ma jednostkę, zakres wiarygodny, normę i zakres ostrzeżenia.
  Opcjonalnie ma też część ciała i regułę punktacji.
* **Składowe pochodne:** punktację snu oblicza domena w `MetricHistory::record` jako sumę
  punktów składowych (`ScoreRule`: liniowo od `zero_at` do `full_at`). Urządzenie nie może jej
  przysłać. Mechanizm jest generyczny i nadaje się dla każdego przyszłego wskaźnika złożonego.
* **`Subject`:** historia należy do astronauty (fizjologia) albo do **stacji** (środowisko).
  Profil astronauty łączy oba źródła (`SnapshotReader`). Alerty środowiskowe widzi więc każdy
  członek załogi stacji.
* **Alerty** (`domain/src/alerts.rs`): `AlertEngine` uruchamia listę `AlertRule` dla każdego
  parametru katalogu.
  * `OutOfRangeRule` generuje alert krytyczny lub ostrzeżenie dla składowych objętych
    `alert_policy`. Każdy nowy parametr dostaje je automatycznie dzięki swoim normom.
  * `OverdueMeasurementRule` generuje alert „warte uwagi”, gdy ostatni pomiar jest starszy
    niż 2 interwały.
  * Nowy rodzaj alertu to nowa implementacja `AlertRule` dopisana w `AlertEngine::standard()`.
  * Ogólny status parametru jest liczony z tych samych składowych co alerty, więc tabela
    załogi i baner zawsze się zgadzają.

### Warstwa HTTP

| Metoda | Ścieżka | Opis |
|---|---|---|
| POST | `/api/auth/login` | `{username, password}` → `{token, expiresAt, user}` |
| GET | `/api/me` | Zalogowany użytkownik |
| GET | `/api/metrics` | Katalog parametrów (zakładka, zakres, składowe, normy, część ciała, punktacja) |
| GET | `/api/metric-groups` | Zakładki w kolejności, z oznaczeniem domyślnej |
| GET | `/api/astronauts` | Załoga stacji ze statusami i licznikami alertów (tylko nadzór) |
| GET | `/api/astronauts/{id}` | Podsumowanie astronauty |
| GET | `/api/astronauts/{id}/overview` | Najnowsze wartości i statusy (wraz ze środowiskiem stacji) |
| GET | `/api/astronauts/{id}/alerts` | Alerty posortowane według ważności, z celem linku (`group`, `metric`, `findings`) |
| GET | `/api/astronauts/{id}/wellbeing-note` | Najnowsza notka o stanie psychicznym (lub `null`) |
| GET | `/api/astronauts/{id}/metrics/{code}/series?from&to&bucket` | Szereg czasowy ze statusem każdego punktu |

### Autoryzacja (JWT)

* Reguły są w jednym miejscu, w `application/src/security.rs` (`AccessPolicy`):
  * **astronauta** widzi wyłącznie swój profil,
  * **nadzorujący** widzi wszystkich astronautów **swojej stacji**, łącznie z ich alertami,
    notkami i zakładkami.
* Reguły są egzekwowane **wewnątrz przypadków użycia**, także nowych (`AlertService`,
  `WellbeingService`). Przechodzą one przez ten sam `Authorizer` co pozostałe, więc żadna
  trasa ani adapter ich nie ominie. Pokrywają to testy w `application/src/services/tests.rs`.
* Dane środowiskowe stacji pobiera się wyłącznie przez profil astronauty, do którego
  użytkownik ma dostęp. Osobnego endpointu stacji nie ma, więc nie ma też osobnej reguły
  do ominięcia.

### Baza danych

Każdy parametr ma **własny agregat historyczny**, czyli fizycznie osobną partycję, tworzoną
automatycznie przy starcie z katalogu (`ensure_storage`), **bez migracji**:

```
measurements          PARTITION BY LIST (metric_code)      -- fizjologia astronauty
  ├── measurements_heart_rate, measurements_body_temperature, measurements_blood_pressure,
  ├── measurements_muscle_atrophy, measurements_bone_density,
  └── measurements_blood_panel, measurements_urine_panel, measurements_sleep
station_measurements  PARTITION BY LIST (metric_code)      -- środowisko stacji (migracja 0002)
  └── station_measurements_air_microbes, _surface_microbes, _radiation, _gravity
latest_measurements / latest_station_measurements          -- projekcja „najnowsza wartość”
wellbeing_notes (astronaut_id, written_at, body)           -- historia notek (migracja 0002)
```

* Klucz główny `(metric_code, subject, recorded_at)` jest jednocześnie indeksem szeregu
  czasowego. Najnowsza wartość to jeden wiersz w projekcji, aktualizowany w tej samej
  transakcji co zapis historii.
* **Badanie laboratoryjne to jeden pomiar** z wieloma składowymi (analitami) w `JSONB`.
  Historia badań to po prostu szereg czasowy tego parametru, a „poprzednie badania” są
  pobierane tym samym generycznym zapytaniem.
* Migracja `0002_environment_alerts_wellbeing.sql` jest addytywna. Sprawdziłem ją na bazie
  z danymi sprzed zmiany.

### Przyjmowanie danych z urządzeń

Port `MeasurementIngestion::ingest(Vec<DeviceMeasurement>)` przyjmuje teraz `subject`, czyli
astronautę albo stację. Domena odrzuca pomiar o niewłaściwym zakresie (np. promieniowanie
przypisane astronaucie) i przysłaną punktację pochodną. Seeder
(`backend/crates/server/src/bin/seed.rs`) generuje historię **każdego** parametru katalogu
z jego `DemoSignal`. Scenariusze (burza słoneczna, zaległe wymazy, gorsze noce, wynik
krytyczny badania) to deklaratywne przesunięcia i pauzy nałożone na ten sygnał. Wszystkie
pomiary przechodzą przez port, jak dane z urządzeń.

---

## Jak dodać nowy parametr zdrowotny

1. **Definicja**: plik w `backend/crates/domain/src/metrics/definitions/`, np.:

```rust
pub fn definition() -> MetricDefinition {
    MetricDefinition::builder("heart_rate", "Tętno spoczynkowe")
        .description("Spoczynkowa częstość akcji serca mierzona rano przez opaskę.")
        .chart(ChartKind::Line)              // line | area | bar | table | body
        .group("physiology")                 // zakładka
        .sampling_hours(24)                  // interwał (także dla alertu o zaległym pomiarze)
        .component(ComponentDefinition::new(
            "resting", "Tętno", "bpm",
            BodyRegion::Heart,                 // część ciała albo None
            Bounds::between(25.0, 220.0),      // zakres wiarygodny (walidacja urządzeń)
            Bounds::between(50.0, 90.0),       // norma
            Bounds::between(45.0, 100.0),      // ostrzeżenie (poza nim: krytyczny)
            0,
            DemoSignal { baseline: 62.0, spread: 5.0, drift_per_day: 0.02, noise: 3.0 },
        ))
        .build()
}
```

   Parametr środowiskowy dostaje dodatkowo `.scope(MetricScope::Station)`. Parametr
   złożony: `.scored(ScoreRule{..})` na składowych i `.derived_score(...)`. Ograniczenie
   alertów do wybranych składowych: `.alert_policy(AlertPolicy::Only(..))`.

2. **Rejestracja**: `pub mod ...;` w `definitions/mod.rs` i jedna linia
   w `MetricCatalog::builtin()`.

Bez dodatkowych kroków działają wtedy: partycja w bazie, walidacja, statusy, **alerty**,
endpointy, widget w odpowiedniej zakładce (domyślny dla typu wykresu, `ScoreWidget` dla
parametrów z punktacją), kolumna statusu zakładki w tabeli załogi i dane demo.
Przykład: commit *„Add resting heart rate metric”* to 1 plik i 2 linie.

Opcjonalnie można dodać dedykowany widget: komponent w `frontend/src/dashboard/widgets/`
i linia w `metricWidgetOverrides` (`frontend/src/dashboard/registry.tsx`). Nowa część ciała
wymaga wariantu `BodyRegion` i kształtu w `PARTS` (`BodyFigure.tsx`).

## Jak dodać zakładkę

Jedna linia `MetricGroup::new("kod", "Nazwa", "opis")` w `MetricCatalog::builtin()`, a potem
`.group("kod")` w definicjach parametrów. Frontend tworzy wpisy bocznego paska i panele
z `/api/metric-groups`. Ikonę dla nowej zakładki można dodać w `GROUP_ICONS`
(`ProfileSidebar.tsx`); bez niej używana jest ikona ogólna.
Moduły niezwiązane z jednym parametrem (jak notka o samopoczuciu) dopisuje się
w `groupWidgets` w rejestrze.

---

## Założenia

* **Środowisko jest per stacja**, nie per astronauta. Profil astronauty pokazuje środowisko
  jego stacji, a alert środowiskowy trafia do każdego członka załogi.
* **Notka o stanie psychicznym** powstaje poza systemem. Tutaj jest tylko odczytywana
  (port `WellbeingRepository`, w seedzie symulowana). Celowo nie ma statusu i nie generuje
  alertów.
* **Punktacja snu** jest sumą sub-punktów (maksymalnie 100). Normy: co najmniej 70 to
  norma, co najmniej 50 to ostrzeżenie.
* **Normy referencyjne** są realistycznymi przybliżeniami (limity ISS dla mikrobów,
  typowe zakresy laboratoryjne, T-score według WHO). To nie są zalecenia kliniczne.
* **Ważność alertów:** `critical` oznacza wartość poza zakresem ostrzeżenia, `warning`
  wartość poza normą, `notice` zaległy pomiar (sprawa warta uwagi, bez ryzyka zdrowotnego).
* Endpoint `/overview` zachowuje pole `body` (mapa wszystkich regionów) dla zgodności API.
  Frontend korzysta teraz z map per parametr.

## Testy

```bash
cd backend && cargo test     # domena (alerty, punktacja, zakres podmiotu), przypadki użycia, JWT
cd frontend && npm test      # Vitest + Testing Library: alerty, tooltip profilu, zakładki
cd frontend && npm run build # typecheck + build produkcyjny
```
