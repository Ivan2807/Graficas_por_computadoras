# Noisy Ditch - Raycasting Survival

Videojuego de supervivencia desarrollado en Rust utilizando Raylib. El proyecto implementa un escenario tipo laberinto generado por habitaciones, renderizado mediante raycasting, enemigos, armas, objetos, llaves y explosiones.

## Características

- Renderizado 3D mediante raycasting.
- Habitaciones generadas aleatoriamente.
- Diferentes niveles de dificultad.
- Enemigos con movimiento y ataque.
- Sistema de vida y escudo.
- Seis armas disponibles:
  - Pistola
  - Escopeta
  - Subfusil
  - Rifle de asalto
  - Rifle de precisión
  - Lanzacohetes
- Desbloqueo progresivo y aleatorio de armas.
- Sistema de munición y recarga.
- Bombas con daño en área.
- Explosiones visibles durante medio segundo.
- Sistema de llaves y habitaciones especiales.
- Mini mapa y radar.
- Enemigo especial tipo mega-monstruo.
- Música y efectos de sonido.

## Controles

| Tecla o botón | Acción |
|---|---|
| `W`, `A`, `S`, `D` | Moverse |
| `Q`, `E` | Girar la cámara |
| Mouse | Apuntar y mover la cámara |
| Click izquierdo | Disparar |
| Click derecho | Abrir o cerrar puertas |
| `1` - `6` | Seleccionar armas desbloqueadas |
| `F` | Lanzar una bomba |
| `R` | Recargar |
| `ESPACIO` | Usar recuperación de vida |
| `ENTER` | Iniciar o reiniciar |

## Objetivo

Sobrevivir, explorar las habitaciones, encontrar las tres llaves y llegar al cuarto final para escapar.

Las armas del 2 al 6 se desbloquean aleatoriamente al entrar a las habitaciones. No se repiten y la pistola está disponible desde el inicio.

## Instalación y ejecución

Clonar el repositorio:

```bash
git clone https://github.com/Ivan2807/Graficas_por_computadoras.git
cd Graficas_por_computadoras/proyecto_1
