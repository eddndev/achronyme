# Plan de Refactorización y Mejora de UX: Herramienta de Series de Fourier

Este documento presenta un plan de acción detallado para la segunda iteración de mejoras en la herramienta de Series de Fourier del proyecto Achronyme.

## 1. Contexto del Proyecto

Achronyme es una suite de herramientas web diseñada para el análisis y la visualización de conceptos clave del Procesamiento Digital de Señales (PDS). Construido sobre un stack tecnológico con Laravel y Livewire en el backend, y Javascript en el frontend, el proyecto busca ofrecer una plataforma interactiva y educativa.

## 2. Estado Actual y Problemática

**Estado Actual:**
Gracias a la refactorización inicial, el proyecto ha alcanzado un hito importante: el cálculo de los coeficientes de Fourier (`a0`, `an`, `bn`) ya **no depende de la API de Wolfram Alpha**. La lógica ahora reside en el backend y utiliza integración numérica, lo que hace que el cálculo sea robusto y fiable.

**Problemática a Resolver:**
Aunque el cálculo es correcto, la experiencia de usuario (UX) es deficiente:
1.  **Visualización Pobre:** La gráfica actual no es clara. Para entender la aproximación de la serie, es fundamental que la función original se muestre en un contexto periódico adecuado.
2.  **Falta de Interactividad:** El slider para seleccionar el número de términos (`n`) no actualiza la gráfica en tiempo real. Requiere una recarga o un nuevo cálculo, lo que rompe la fluidez de la herramienta.
3.  **Código Heredado:** Aún existe el código que realiza la llamada a la API de Wolfram para obtener el `MathML`, el cual se ha decidido retirar.

## 3. Objetivos de la Nueva Iteración

1.  **Mejorar la Visualización:** La gráfica deberá mostrar la **función original a lo largo de dos de sus períodos**, haciendo evidente su naturaleza periódica y mejorando la comprensión de la aproximación de la serie.
2.  **Interactividad en Tiempo Real:** El slider que controla el número de términos (`n`) deberá **actualizar la gráfica de la serie de Fourier instantáneamente**, sin recálculos costosos en el backend.
3.  **Simplificación Definitiva:** **Eliminar por completo la dependencia y la llamada a la API de Wolfram Alpha**, limpiando el código de lógica innecesaria.

---

## 4. Flujo de Tareas

### Tarea 1: Simplificar el Backend (`app/Livewire/FourierSeriesTool.php`)

**Objetivo:** Limpiar el código heredado y asegurar que el backend envíe al frontend toda la información necesaria para la nueva visualización interactiva.

#### 1.1. Código a Eliminar
Dentro del método `calculate()`:
*   **Elimina por completo el bucle `foreach ($queries as $key => $query)`**. Su único propósito era llamar a la API de Wolfram, lo cual ya no es necesario.
*   **Elimina las propiedades de la clase** que almacenaban los resultados de Wolfram, como `public array $debugOutput = [];`.

#### 1.2. Código a Modificar y Verificar
Dentro del método `calculate()`:

1.  **Asegurar Cálculo de 50 Términos:** Confirma que el bucle que calcula `an` y `bn` siempre se ejecute hasta el máximo de términos (50), independientemente del valor del slider.

    ```php
    // En app/Livewire/FourierSeriesTool.php, dentro de calculate()
    $N = 50; // Usar siempre el valor máximo

    for ($n = 1; $n <= $N; $n++) {
        // ... lógica de cálculo de an y bn ...
    }
    ```

2.  **Enviar Datos Clave al Frontend:** Modifica el array final `$this->seriesCoeffs` para incluir el período y el inicio del dominio. El frontend los necesita para la gráfica de 2 períodos.

    ```php
    // Al final del método calculate()
    $this->seriesCoeffs = [
        'a0' => $this->evaluated_a0,
        'an' => $this->evaluated_an, // Array con 50 coeficientes
        'bn' => $this->evaluated_bn, // Array con 50 coeficientes
        'period' => $T_val,          // <-- AÑADIR/VERIFICAR
        'domainStart' => $a_val,     // <-- AÑADIR/VERIFICAR
    ];
    ```

### Tarea 2: Implementar Interactividad en el Frontend

**Objetivo:** Hacer que la interfaz sea fluida, interactiva y visualmente informativa.

#### 2.1. Conectar el Slider (en `resources/views/livewire/fourier-series-tool.blade.php`)

*   Localiza el `<input type="range">` que controla el número de términos.
*   Añádele el atributo `wire:model.live="terms_n"`. Esto sincronizará el valor del slider con el backend en tiempo real.

    ```html
    <input type="range" wire:model.live="terms_n" min="1" max="50" />
    ```

#### 2.2. Adaptar el JavaScript (en `resources/js/fs/app.js`)

1.  **Graficar 2 Períodos de la Función Original:**
    *   En tu JS, lee los nuevos valores `period` y `domainStart` que vienen del backend.
    *   Genera los puntos `(x, y)` para la gráfica de la función original en el rango `[domainStart, domainStart + 2 * period]`.

    ```javascript
    // Lógica conceptual en tu JS
    const a = data.seriesCoeffs.domainStart;
    const T = data.seriesCoeffs.period;
    const plotDomainEnd = a + 2 * T; // Graficar sobre 2 períodos

    // Genera los puntos para la traza de la función original usando este nuevo dominio
    // ...
    ```

2.  **Reconstruir la Serie Dinámicamente:**
    *   La función que dibuja la serie de Fourier debe usar el valor actual del slider (`data.terms_n`) como límite superior del bucle de suma.

    ```javascript
    // Lógica conceptual en tu JS para reconstruir la serie
    function redrawFourierSeries(data) {
        const a0 = data.seriesCoeffs.a0;
        const an = data.seriesCoeffs.an; // Array completo de 50 coeficientes
        const bn = data.seriesCoeffs.bn; // Array completo de 50 coeficientes
        const limit = data.terms_n;      // Límite actual del slider

        // ...
        let y = a0 / 2;
        // El bucle se detiene en 'limit', no en la longitud total del array
        for (let n = 1; n <= limit; n++) {
            // Accede a los coeficientes (ej. an[n] si es un objeto, o an[n-1] si es array)
            y += an[n] * Math.cos(...) + bn[n] * Math.sin(...);
        }
        // ...
    }
    ```

3.  **Activar el Redibujado Automático:**
    *   Usa un hook de Livewire para escuchar los cambios y volver a dibujar la gráfica sin recargar la página.

    ```javascript
    // Añadir esto a tu JS
    Livewire.hook('component:init', ({ component, cleanup }) => {
        // Dibujo inicial cuando el componente se carga
        redrawFourierSeries(component.snapshot.data);

        // Escucha cada actualización del componente (ej. al mover el slider)
        component.on('updated', () => {
            // Vuelve a dibujar con los datos actualizados (nuevo `terms_n`)
            redrawFourierSeries(component.snapshot.data);
        });
    });
    ```
