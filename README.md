# Proyecto 1 - Rust Raycaster

Motor de *raycasting* estilo Wolfenstein 3D escrito en Rust con [raylib](https://www.raylib.com/).
Proyecto del curso de Gráficas por Computadora (UVG).

La escena 3D se genera lanzando un rayo por cada columna de pantalla sobre un mapa
de rejilla 16x16, usando el algoritmo DDA para encontrar la pared más cercana y
dibujando una línea vertical cuya altura depende de la distancia corregida
(*fisheye correction*).

## Características

- **Renderizado por raycasting** con DDA, corrección de ojo de pez y sombreado por distancia.
- **Paredes de varios tipos** (valores 1-4 en el mapa), cada uno con su color; las caras
  norte/sur se oscurecen para dar sensación de volumen.
- **Sprites animados** con proyección a pantalla, ordenamiento por distancia y oclusión
  correcta contra las paredes mediante un *z-buffer* por columna.
- **Minimapa** en la esquina superior derecha con la posición y dirección del jugador.
- **Movimiento con colisión** (WASD + rotación con mouse), radio de colisión y deslizamiento
  por eje para no atravesar paredes.
- **Máquina de estados**: pantalla de bienvenida → selección de nivel → juego → pantalla de éxito.
- **Dos niveles** con distinto layout, posición inicial y casilla de meta.
- **Disparo** con click izquierdo o barra espaciadora: elimina el sprite vivo más cercano
  dentro de un cono de 0.15 rad y 10 unidades de rango.
- Contador de FPS en pantalla (objetivo de 60 FPS con vsync).

## Requisitos

- Rust (edición 2024, por lo que se necesita un toolchain reciente: `rustup update`).
- Dependencias de compilación de raylib. En Ubuntu/Debian:

  ```bash
  sudo apt install build-essential cmake libasound2-dev libx11-dev libxrandr-dev \
      libxi-dev libgl1-mesa-dev libglu1-mesa-dev libxcursor-dev libxinerama-dev
  ```

## Cómo ejecutarlo

```bash
cargo run --release
```

El modo `--release` está configurado con `opt-level = 3` y LTO; en modo debug el
raycasting va notablemente más lento.

## Controles

| Acción | Tecla |
|---|---|
| Avanzar / retroceder | `W` / `S` |
| Desplazamiento lateral | `A` / `D` |
| Rotar la cámara | Mouse |
| Disparar | Click izquierdo o `Espacio` |
| Continuar (bienvenida / éxito) | `Enter` o `Espacio` |
| Navegar el menú de niveles | Flechas `↑` `↓` + `Enter` |
| Volver al menú desde el juego | `Esc` |

El cursor se captura al entrar a un nivel y se libera al salir con `Esc` o al completarlo.

## Estructura del código

```
src/
├── main.rs        # Bucle principal, máquina de estados, input, disparo y pantallas de UI
├── state.rs       # Enum GameState (Welcome, LevelSelect, Playing, Success)
├── map.rs         # Rejillas de los niveles, struct Level y consultas de colisión/tipo de pared
├── player.rs      # Posición, ángulo, movimiento con colisión y rotación del jugador
├── raycasting.rs  # DDA, colores de pared, render de la escena 3D y del minimapa
└── sprite.rs      # AnimatedSprite y su proyección a pantalla con z-buffer
```

### Cómo agregar o editar un nivel

Los mapas son matrices `[[u8; 16]; 16]` en `src/map.rs`. `0` es piso libre y `1`-`4`
son los tipos de pared (cada número da un color distinto, ver `wall_color` en
`src/raycasting.rs`). Para agregar un nivel:

1. Define una nueva constante siguiendo el patrón de `LEVEL_1` / `LEVEL_2`.
2. Agrégala al `Vec<Level>` que devuelve `get_levels()`, indicando `name`,
   `player_start`, `player_start_angle` y `goal_tile` (la casilla `(col, fila)` que
   dispara la pantalla de éxito al pisarla).
3. Si quieres sprites propios para ese nivel, añádelo al `match` de
   `spawn_sprites_for_level()` en `src/main.rs`.

Mantén el borde exterior con paredes: fuera del mapa se trata como pared, pero el
borde evita que el jugador vea "hacia afuera".

## Pendientes

- Música de fondo y efecto de sonido al disparar (hay un `TODO` con el esqueleto
  usando `RaylibAudio` en `src/main.rs`).
- Texturas para paredes y sprites: hoy se dibujan con colores planos.
