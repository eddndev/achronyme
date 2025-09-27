<div align="center">
  <img src="./assets/logo.svg" alt="Logo del Proyecto" width="200">
  <br/>
  <h1>
    <b>Achronyme: Herramientas para Procesamiento Digital de Señales</b>
  </h1>
  <p>
    Una suite de herramientas web para el análisis y visualización de conceptos clave del Procesamiento Digital de Señales (PDS), incluyendo la Transformada de Fourier, Series de Fourier, Convolución y más.
  </p>
</div>

<div align="center">
  <img src="https://img.shields.io/badge/PHP-%3E%3D8.2-8A2BE2?style=for-the-badge&logo=php&logoColor=white" alt="PHP version requirement">
  <img src="https://img.shields.io/badge/Laravel-12.x-FF2D20?style=for-the-badge&logo=laravel&logoColor=white" alt="Laravel">
  <img src="https://img.shields.io/badge/Vite-646CFF?style=for-the-badge&logo=vite&logoColor=white" alt="Vite">
  <img src="https://img.shields.io/badge/Tailwind_CSS-9370DB?style=for-the-badge&logo=tailwind-css&logoColor=white" alt="Tailwind CSS">
  <img src="https://img.shields.io/badge/License-MIT-9370DB?style=for-the-badge" alt="License MIT">
</div>

---

## 📜 Descripción del Proyecto

Este proyecto nace como una herramienta educativa y de análisis para la materia de **Procesamiento Digital de Señales**. Su propósito es ofrecer una plataforma interactiva donde los usuarios puedan experimentar con diferentes algoritmos y visualizar los resultados en tiempo real.

La aplicación busca sintetizar las herramientas fundamentales del PDS, permitiendo a estudiantes y profesionales definir señales, aplicar transformaciones y analizar sus propiedades tanto en el dominio del tiempo como en el de la frecuencia.

---

## ✨ Herramientas Incluidas

* **Transformada de Fourier:** Calcula y visualiza el espectro de magnitud y fase de una señal.
* **Serie de Fourier:** Analiza funciones periódicas descomponiéndolas en una suma de senos y cosenos.
* **Convolución:** Permite visualizar la convolución de dos señales en tiempo real.
* **Y más:** Se planea incluir herramientas adicionales en futuras actualizaciones.

---

## 🛠️ Tecnologías Utilizadas

Este proyecto está construido sobre una arquitectura moderna utilizando:

* **Backend:**
    * **Laravel 12:** Como framework principal para la lógica de negocio y el enrutamiento.
    * **PHP 8.2:** El lenguaje de programación del lado del servidor.
* **Frontend:**
    * **Vite:** Como servidor de desarrollo y empaquetador de assets.
    * **Tailwind CSS:** Para un estilizado rápido y mantenible.
    * **GSAP:** Para la creación de animaciones fluidas y de alto rendimiento.
    * **Plotly.js:** Para la renderización de las gráficas interactivas.
    * **MathJax:** Para el renderizado de alta calidad de las fórmulas LaTeX.

---

## 🚀 Instalación y Puesta en Marcha

Para ejecutar este proyecto en tu entorno local, sigue estos sencillos pasos.

### **Pre-requisitos**

Asegúrate de tener instalado **PHP 8.2+**, **Composer** y **Node.js**.

### **Pasos de Instalación**

1.  **Clona el repositorio** en tu máquina local:

    ```bash
    git clone https://github.com/eddndev/processignals.git
    ```

2.  **Navega al directorio** del proyecto:

    ```bash
    cd processignals
    ```

3.  **Instala las dependencias de PHP** con Composer:

    ```bash
    composer install
    ```

4.  **Instala las dependencias de Node.js** con npm:

    ```bash
    npm install
    ```

5.  **Crea una copia del archivo de entorno** y configúralo:

    ```bash
    cp .env.example .env
    ```
    *Abre el archivo `.env` y configura tu base de datos y otras variables de entorno necesarias.*

6.  **Genera la clave de la aplicación** de Laravel:

    ```bash
    php artisan key:generate
    ```

7.  **Ejecuta las migraciones** para crear las tablas de la base de datos:
    ```bash
    php artisan migrate
    ```

8.  **Inicia el servidor de desarrollo** de Laravel y Vite:

    ```bash
    # En una terminal
    php artisan serve

    # En otra terminal
    npm run dev
    ```

¡Y listo\! La aplicación estará disponible en `http://localhost:8000`.

-----

## 👥 Equipo

Este proyecto fue desarrollado por el **Equipo 1 - Procesamiento Digital de Señales**.

| \# | Integrante                    |
| :-: | :---------------------------- |
| 1 | `Alonso Sánchez Eduardo`        |
| 2 | `Bonilla Ramírez Josué Eleazar` |
| 3 | `Jiménez Meza Ana Harumi`       |
| 4 | `Quiroz Mora Abel Mauricio`     |
| 5 | `Vilchis Paniagua Johan Emiliano` |

-----

## 📄 Licencia

Este proyecto está bajo la Licencia MIT. Consulta el archivo `LICENSE` para más detalles.
