Proyecto 2

// Video https://youtu.be/zTifaYywCyA
La escena: auditorio ITESO
Geometría (~180 objetos)
Cuarto: 6 planos (piso, techo, 4 paredes) de 12 × 6 × 20 m

Escenario elevado: caja de 11.5 × 1.0 × 7.0 m al fondo

Panel azul del fondo: caja de 5 × 3.5 × 0.4 m (el objeto que se pinta por defecto)

Slats de madera: 28 cajas horizontales a los lados del panel

Podium y mesa del presídium sobre el escenario

Gradería: 6 escalones de 0.4 m de subida cada uno

Sillas: 6 filas × 6 columnas, cada una con asiento + respaldo

Ventanas laterales: 12 cajas delgadas con vidrio celeste

Estrellas: 72 esferas pequeñas frente a las ventanas

Focos del techo: 12 esferas emisivas

Iluminación
12 luces puntuales, cada una asociada a una fila (0–3) y a un estado enabled

Sol direccional cuya elevación y color dependen del ciclo día/noche

Cambio automático de las ventanas (azul cielo ↔ negro) y las estrellas (invisibles ↔ blancas) al cruzar el umbral día/noche

Interacción
Entrada	Acción
Flechas	Rotar la cámara (yaw/pitch)
Click izquierdo	Pintar el objeto bajo el cursor
1–9	Elegir color (rojo, naranja, amarillo, verde, cian, azul, púrpura, rosa, blanco)
Q W E R T Y	Elegir tipo de material (Difuso, Brillante, Plástico, Metal, Emisivo, Espejo)
A S D F	Toggle fila de luces 0/1/2/3
G	Toggle todas las luces (si todas están on → apaga; si no → enciende)
ESPACIO	Alternar entre día y noche (transición suave)
El cursor cambia de color según si apunta a un objeto pintable (verde) o al vacío (blanco).

Efecto "pintura fresca"
Al pintar, el objeto recibe primero un material húmedo (más especular, shininess alto, albedo ligeramente más oscuro, como cuando se moja una superficie). Con el tiempo interpola hacia el material seco (el elegido con QWERTY) con una curva ease-out.

Cada tipo de material tiene su propia duración y factor de humedad:

Material	Duración base	Factor de humedad
Espejo	1.5 s	0.15
Metal	2.0 s	0.25
Emisivo	3.0 s	0.15
Brillante	2.5 s	0.40
Plástico	3.5 s	0.60
Difuso	4.5 s	0.85
Los colores más oscuros tardan hasta 60% más en secar (más pigmento). Mientras mantengas el click, la pintura se mantiene fresca.

Ciclo día/noche
sun_t interpola entre 0 (noche) y 1 (día). Al presionar ESPACIO se cambia el objetivo y sun_t se mueve hacia él a velocidad constante.

Dirección del sol: (0.25, sin(sun_t · π/2), −cos(sun_t · π/2)) — el sol baja hasta el horizonte de noche

Intensidad: sun_t² — el amanecer/atardecer se atenúa suavemente

Color: blanco al mediodía, naranja cerca del horizonte

Ambient: sube de 0.06 a 0.30

Ventanas: azul cielo → negro; estrellas: invisibles → blancas emisivas

Las ventanas y estrellas se actualizan solo cuando cruzan el umbral día/noche, no cada frame.

Optimizaciones
Render a 0.5× resolución (400×300) y escalado a 800×600 con draw_texture_pro + filtro bilineal

Salto de luces apagadas en el shading → si todas están off, el shading de luces puntuales es gratis

needs_render: solo se redibuja cuando algo cambia (input, pintura secándose, transición de sol)

Actualización día/noche por transición: las ~84 ventanas/estrellas se tocan dos veces por cambio de estado, no cada frame
