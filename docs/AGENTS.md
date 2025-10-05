# Estándar de Gestión de Proyecto y Documentación

**Versión:** 1.1
**Última Actualización:** 2025-10-04
**Propósito:** Este documento describe la metodología, herramientas y flujos de trabajo a seguir durante el ciclo de vida del desarrollo. Su objetivo es asegurar la consistencia, claridad y trazabilidad en todas las fases.

**⚠️ IMPORTANTE: Esta metodología es INQUEBRANTABLE e INAMOVIBLE.**

---

## ⚠️ REGLAS INQUEBRANTABLES

**Esta metodología es OBLIGATORIA e INAMOVIBLE. Los agentes DEBEN seguir estas reglas sin excepción:**

### 1. Issues y Épicas
- ✅ **SIEMPRE** crear issues usando la plantilla definida en la sección C (Descripción + Criterios de Aceptación + Tareas Técnicas)
- ✅ **SIEMPRE** usar el sistema de labels correcto (Type, Module, Priority, Sprint)
- ❌ **NUNCA** añadir Épicas (label `Epic`) al proyecto Kanban
- ❌ **NUNCA** crear issues sin criterios de aceptación medibles
- ❌ **NUNCA** crear issues genéricas sin detallar las tareas técnicas

### 2. GitHub Projects
- ✅ **SIEMPRE** crear un GitHub Project para cada repositorio nuevo
- ✅ **SIEMPRE** configurar workflows de automatización usando la plantilla en sección 5.E
- ✅ **SIEMPRE** mover issues manualmente a "Todo" si los workflows no están activos
- ❌ **NUNCA** dejar issues en "No Status" sin razón
- ❌ **NUNCA** tener más de 1 tarea en "In Progress" por desarrollador

### 3. Sprints
- ✅ **SIEMPRE** crear una Épica Maestra para cada sprint
- ✅ **SIEMPRE** crear y actualizar el diario del sprint en `/docs/sprints/`
- ✅ **SIEMPRE** documentar decisiones técnicas y bloqueos en tiempo real
- ❌ **NUNCA** empezar un sprint sin su documento en `/docs/sprints/`
- ❌ **NUNCA** cerrar un sprint sin completar la retrospectiva

### 4. Documentación
- ✅ **SIEMPRE** aplicar filosofía "Docs-First": documentar ANTES de implementar
- ✅ **SIEMPRE** actualizar `/docs/` cuando cambien decisiones de arquitectura
- ❌ **NUNCA** crear código sin documentación previa en `/docs/`
- ❌ **NUNCA** dejar documentación desactualizada

### 5. GitHub CLI y Automatización
- ✅ **SIEMPRE** verificar scopes de autenticación (`project`, `read:project`) antes de crear proyectos
- ✅ **SIEMPRE** crear un Personal Access Token (PAT) con scopes `repo` y `project` para workflows
- ✅ **SIEMPRE** agregar el PAT como secret `PROJECT_PAT` en el repositorio
- ✅ **SIEMPRE** guardar IDs del proyecto (PROJECT_ID, STATUS_FIELD_ID, etc.) en variables de entorno del workflow
- ✅ **SIEMPRE** usar la checklist de la sección 5.F al configurar un proyecto nuevo
- ❌ **NUNCA** crear proyectos sin workflows de automatización
- ❌ **NUNCA** usar solo GITHUB_TOKEN para Projects (no tiene permisos suficientes)

**Si un agente rompe alguna de estas reglas, debe detenerse y corregir inmediatamente antes de continuar.**

---

## 1. Filosofía General

1.  **Docs-First (La Documentación Primero):** La planificación y documentación preceden a la implementación. Se definen el "qué", "porqué" y "cómo" antes de escribir código.
2.  **GitHub como Única Fuente de Verdad (SSOT):** Todo lo relacionado con el proyecto —código, documentación, tareas y discusiones— vive dentro del ecosistema de GitHub.
3.  **Claridad sobre Brevedad:** Las descripciones de tareas (Issues) deben ser explícitas, detallando el objetivo y los criterios de aceptación para eliminar suposiciones.
4.  **Documentación Viva:** Los documentos en `/docs/` son dinámicos y se actualizan si las decisiones de arquitectura o diseño cambian.

---

## 2. Estructura de Documentación (`/docs/`)

La carpeta `/docs/` es el cerebro del proyecto.

* **`/docs/01-manifest.md`**: La "Constitución" del proyecto. Define la visión, objetivos y alcance.
* **`/docs/02-design-system.md`**: La fuente de verdad para la UI. Traduce Figma a tokens de diseño.
* **`/docs/03-database-schema.md`**: El plano arquitectónico de los datos, con un diagrama Mermaid.
* **`/docs/04-user-stories.md`**: El backlog completo de funcionalidades. Es la fuente para crear Issues.
* **`/docs/sprints/[sprint-number]-[sprint-name].md`**: La bitácora de cada sprint. Registra el alcance, decisiones, bloqueos y resultados.

---

## 3. Flujo de Trabajo en GitHub

### A. GitHub Projects (Kanban)

El tablero Kanban es la representación visual del trabajo.

* `Backlog`: Almacén de todas las tareas (Issues) no priorizadas para el sprint actual.
* `To Do (Sprint Actual)`: Tareas seleccionadas y comprometidas para el sprint en curso.
* `In Progress`: La tarea en la que se está trabajando activamente.
* `Done`: Tareas completadas y cerradas.

### B. Estructura de Issues: Épicas y Tareas

El trabajo se organiza en dos niveles:

1.  **Épicas (Issues "Padre"):**
    * **Qué son:** Issues de alto nivel que agrupan un conjunto de funcionalidades (ej. `Épica: Módulo 2 - Gestión de Citas`).
    * **Cómo se identifican:** Usan la etiqueta `Epic` 📚.
    * **Propósito:** Agrupar y ver el progreso general de un área a través de su checklist de tareas. **No se añaden al Kanban.**

2.  **Tareas (Sub-issues / Historias de Usuario):**
    * **Qué son:** Issues accionables que implementan una única historia de usuario (ej. `[Citas] Agendar una nueva cita`).
    * **Cómo se identifican:** Usan etiquetas de `Type`, `Module` y `Priority`.
    * **Propósito:** Es el trabajo real. **Estas son las issues que se añaden al Kanban** y se mueven a través de las columnas.

### C. La Plantilla de Issue Maestra

Cada `sub-issue` de tipo Tarea **debe** seguir esta estructura en su descripción:

```markdown
**Descripción:**
> [Aquí se pega una descripción para identificar el propósito de la tarea."]

---

### Criterios de Aceptación (AC)
*Se considera completada esta tarea CUANDO:*
- [ ] Criterio 1 (Define una condición medible y verificable).
- [ ] Criterio 2 (Ej: La acción A resulta en el efecto B).
- [ ] Criterio 3 (Ej: El usuario ve el mensaje C).

---

### Tareas Técnicas (Checklist de Implementación)
- [ ] Tarea técnica 1 (Ej: Crear el archivo de migración...).
- [ ] Tarea técnica 2 (Ej: Implementar el método X en el Controlador Y).
- [ ] Tarea técnica 3 (Ej: Escribir la prueba de funcionalidad Z).
````

### D. Sistema de Etiquetas (Labels)

Un sistema de etiquetas consistente es crucial para la organización.

#### Tipo de Tarea (Type)

*Indica la naturaleza del trabajo.*

  * `Type: Feature`: Nueva funcionalidad para el usuario.
  * `Type: Chore`: Tareas de mantenimiento, configuración o refactorización.
  * `Type: Bug`: Corrección de un error en el código existente.
  * `Type: Documentation`: Creación o actualización de la documentación.

#### Módulo de la Aplicación (Module)

*Asocia la tarea a un área específica del proyecto.*

  * `Module: Auth`: Autenticación, registro, perfiles de usuario.
  * `Module: Database`: Migraciones, seeders, cambios de esquema.
  * `Module: UI/UX`: Tareas de frontend, estilos, componentes visuales.
  * `Module: Citas`: Flujo de reserva, calendario, gestión de citas.
  * `Module: Lealtad`: Sellos, recompensas, cupones.
  * `Module: Admin (Nova)`: Tareas relacionadas con el panel de Laravel Nova.
  * `Module: Core`: Lógica de negocio principal, modelos, servicios.

#### Prioridad (Priority)

*Indica la urgencia de la tarea.*

  * `Priority: Critical` 🔴: Debe ser atendido inmediatamente.
  * `Priority: High` 🟠: Importante, para el sprint actual si es posible.
  * `Priority: Medium` 🟡: Prioridad normal.
  * `Priority: Low` 🟢: Tarea de baja urgencia.

#### Etiquetas Especiales

  * `Epic` 📚: Para identificar las Issues "Padre" que agrupan tareas.
  * `Sprint: 1`, `Sprint: 2`, etc. 🚀: Para agrupar todas las tareas de un sprint.
  * `Status: Blocked` 🚧: La tarea no puede continuar.
  * `Status: Needs Review` 👀: La tarea está completa y tiene un Pull Request abierto.

-----

## 4\. Ciclo de Vida de un Sprint

Un sprint es un ciclo de trabajo con un objetivo claro.

1.  **Planificación del Sprint:**

      * Se revisa el `Backlog` del Kanban y `/docs/04-user-stories.md`.
      * Se seleccionan las `Tareas` para el nuevo sprint.
      * Se crea una `Épica Maestra` para el sprint, enlazando todas las `Tareas` seleccionadas.
      * Se mueven las `Tareas` seleccionadas del `Backlog` a la columna `To Do (Sprint Actual)`.

2.  **Creación del Diario del Sprint:**

      * Se crea un nuevo archivo en `/docs/sprints/` (ej. `02-appointment-booking.md`).
      * El archivo **debe** crearse utilizando la siguiente plantilla para asegurar la consistencia:

      ```markdown
      # Diario del Sprint [Número]: [Nombre del Sprint]

      **Periodo:** [Fecha de Inicio] - [Fecha de Fin]

      **Épica Maestra en GitHub:** [Enlace a la Épica Maestra del Sprint]

      ---

      ## 1. Objetivo del Sprint

      [Describe aquí el objetivo principal y el resultado esperado del sprint en 1-2 frases.]

      ## 2. Alcance y Tareas Incluidas

      [Lista aquí, en formato de checklist, todas las issues/tareas que forman parte del sprint.]

      - [ ] `#IssueID - [Título de la Issue]`
      - [ ] `#IssueID - [Título de la Issue]`

      ---

      ## 3. Registro de Decisiones Técnicas

      *Esta sección es un log vivo. Se actualiza a medida que se toman decisiones durante el sprint.*

      *   **[FECHA]:** [Descripción de la decisión técnica.]
          *   **Razón:** [Justificación clara y concisa de por qué se tomó esa decisión.]

      ---

      ## 4. Registro de Bloqueos y Soluciones

      *Esta sección documenta los problemas inesperados y cómo se resolvieron.*

      *   **[FECHA]:**
          *   **Problema:** [Descripción del bloqueo.]
          *   **Solución:** [Cómo se resolvió el problema.]

      ---

      ## 5. Resultado del Sprint (A completar al final)

      *   **Tareas Completadas:** [ ] X de Y
      *   **Resumen:** [Escribe un resumen ejecutivo del resultado del sprint. ¿Se cumplió el objetivo?]
      *   **Aprendizajes / Retrospectiva:**
          *   **Qué funcionó bien:** [Anota los puntos positivos y las prácticas exitosas.]
          *   **Qué se puede mejorar:** [Identifica áreas de mejora para futuros sprints.]
      ```

3.  **Ejecución del Sprint:**

      * Se mueven las `Tareas` a través del Kanban.
      * Se documentan en vivo las decisiones y bloqueos en el diario del sprint.

4.  **Cierre y Retrospectiva:**

      * Al completar todas las `Tareas`, se cierra la `Épica Maestra`.
      * Se completa la sección "Resultado del Sprint" en el diario, anotando resúmenes y aprendizajes.

---

## 5. GitHub Projects - Guía para Agentes

### Propósito

Esta sección documenta cómo los agentes deben crear, configurar y gestionar GitHub Projects para cualquier proyecto que siga esta metodología.

### A. Verificación de Autenticación

Antes de trabajar con Projects, verificar que GitHub CLI tiene los scopes necesarios:

```bash
# Verificar autenticación
gh auth status

# Si faltan scopes de project, actualizar:
gh auth refresh -h github.com -s project,read:project
```

**Scopes requeridos:** `project`, `read:project`

### A.1 Crear Personal Access Token (PAT) para Workflows

**⚠️ CRÍTICO:** Los workflows de GitHub Actions NO pueden usar `GITHUB_TOKEN` para Projects porque no tiene permisos suficientes.

**Pasos para crear el PAT:**

1. Ir a: https://github.com/settings/tokens
2. Click "Generate new token" → "Generate new token (classic)"
3. Nombre: `PROJECT_AUTOMATION`
4. Seleccionar scopes:
   - ✅ `repo` (Full control of private repositories)
   - ✅ `project` (Full control of projects)
5. Click "Generate token"
6. **COPIAR EL TOKEN** (solo se muestra una vez)

**Agregar el PAT como secret:**

1. Ir a: `https://github.com/[OWNER]/[REPO]/settings/secrets/actions`
2. Click "New repository secret"
3. Name: `PROJECT_PAT`
4. Secret: pegar el token copiado
5. Click "Add secret"

El workflow ya está configurado para usar `${{ secrets.PROJECT_PAT || secrets.GITHUB_TOKEN }}` automáticamente.

### B. Crear un Nuevo GitHub Project

**Paso 1: Crear el proyecto**

```bash
gh project create --owner [USERNAME] --title "[NOMBRE DEL PROYECTO]" --format json
```

Esto devuelve un JSON con:
- `id`: ID del proyecto (formato: `PVT_xxxxx`) - **GUARDAR ESTE ID**
- `url`: URL del proyecto
- `number`: Número del proyecto

**Paso 2: Extraer IDs importantes**

```bash
# Listar campos del proyecto para obtener IDs
gh project field-list [PROJECT_NUMBER] --owner [USERNAME] --format json --limit 20
```

Buscar en el output:
- `Status` field → guardar su `id` (PVTSSF_xxxxx)
- Opciones de Status → guardar IDs de: `Todo`, `In Progress`, `Done`

**Ejemplo de output:**
```json
{
  "id": "PVTSSF_xxxxx",
  "name": "Status",
  "options": [
    {"id": "f75ad846", "name": "Todo"},
    {"id": "47fc9ee4", "name": "In Progress"},
    {"id": "98236657", "name": "Done"}
  ]
}
```

### C. Añadir Issues al Proyecto

**Añadir una issue individual:**

```bash
gh project item-add [PROJECT_NUMBER] --owner [USERNAME] --url https://github.com/[OWNER]/[REPO]/issues/[NUM]
```

**Añadir múltiples issues (loop):**

```bash
for issue in {START..END}; do
  gh project item-add [PROJECT_NUMBER] --owner [USERNAME] --url "https://github.com/[OWNER]/[REPO]/issues/$issue"
  echo "Issue #$issue agregada"
done
```

**Regla importante:** Las issues con label `Epic` NO deben añadirse al proyecto Kanban (solo sirven para tracking).

### D. Mover Issues entre Columnas

Para mover issues entre columnas, usar GraphQL API:

**Paso 1: Obtener el Item ID de la issue en el proyecto**

```bash
gh api graphql -f query='
  query($issueNodeId: ID!, $projectId: ID!) {
    node(id: $projectId) {
      ... on ProjectV2 {
        items(first: 100) {
          nodes {
            id
            content {
              ... on Issue {
                id
              }
            }
          }
        }
      }
    }
  }' -f issueNodeId="[ISSUE_NODE_ID]" \
     -f projectId="[PROJECT_ID]" \
     --jq '.data.node.items.nodes[] | select(.content.id == "[ISSUE_NODE_ID]") | .id'
```

**Paso 2: Actualizar el campo Status**

```bash
gh api graphql -f query='
  mutation($projectId: ID!, $itemId: ID!, $fieldId: ID!, $optionId: String!) {
    updateProjectV2ItemFieldValue(input: {
      projectId: $projectId
      itemId: $itemId
      fieldId: $fieldId
      value: {singleSelectOptionId: $optionId}
    }) {
      projectV2Item {
        id
      }
    }
  }' -f projectId="[PROJECT_ID]" \
     -f itemId="[ITEM_ID]" \
     -f fieldId="[STATUS_FIELD_ID]" \
     -f optionId="[OPTION_ID]"
```

Donde `[OPTION_ID]` es:
- `TODO_OPTION_ID` para mover a "Todo"
- `IN_PROGRESS_OPTION_ID` para mover a "In Progress"
- `DONE_OPTION_ID` para mover a "Done"

### E. Crear Workflows de Automatización

El agente debe crear workflows en `.github/workflows/project-board-automation.yml` con las siguientes automatizaciones:

#### Automatizaciones Recomendadas:

1. **Auto-añadir issues con label Sprint:**
   - Trigger: `issues.opened` + tiene label `Sprint: X`
   - Acción: Añadir al proyecto

2. **Auto-mover a Todo:**
   - Trigger: `issues.opened` o `issues.reopened`
   - Condición: NO tiene label `Epic`
   - Acción: Mover a columna "Todo"

3. **Auto-mover a In Progress:**
   - Trigger: `issues.assigned` O `issues.labeled` (label: `Status: In Progress`)
   - Condición: NO tiene label `Epic`
   - Acción: Mover a columna "In Progress"

4. **Auto-mover a Done:**
   - Trigger: `issues.closed`
   - Condición: NO tiene label `Epic`
   - Acción: Mover a columna "Done"

5. **Marcar PR como Needs Review:**
   - Trigger: `pull_request.opened`
   - Acción: Añadir label `Status: Needs Review` a issues vinculadas

#### Template de Workflow:

```yaml
name: Project Board Automation

on:
  issues:
    types: [opened, labeled, assigned, closed, reopened]
  pull_request:
    types: [opened, ready_for_review]

env:
  PROJECT_ID: [GUARDAR_AQUI]
  STATUS_FIELD_ID: [GUARDAR_AQUI]
  TODO_OPTION_ID: [GUARDAR_AQUI]
  IN_PROGRESS_OPTION_ID: [GUARDAR_AQUI]
  DONE_OPTION_ID: [GUARDAR_AQUI]

jobs:
  # Ver archivo completo en .github/workflows/project-board-automation.yml
```

### F. Flujo de Trabajo del Agente al Configurar un Proyecto Nuevo

**Checklist para el agente:**

1. [ ] Verificar autenticación de GitHub CLI con scopes `project` y `read:project`
2. [ ] **CREAR Personal Access Token (PAT)** con scopes `repo` y `project`
3. [ ] **AGREGAR el PAT como secret `PROJECT_PAT`** en settings del repositorio
4. [ ] Crear el GitHub Project con `gh project create`
5. [ ] Guardar el `PROJECT_ID` del JSON resultante
6. [ ] Obtener `STATUS_FIELD_ID` y los IDs de opciones (Todo, In Progress, Done) con `gh project field-list`
7. [ ] Guardar todos los IDs en variables de entorno del workflow
8. [ ] Crear archivo `.github/workflows/project-board-automation.yml` con las automatizaciones
9. [ ] Verificar que el workflow usa `${{ secrets.PROJECT_PAT || secrets.GITHUB_TOKEN }}`
10. [ ] Añadir todas las issues del sprint actual al proyecto con `gh project item-add`
11. [ ] Mover issues desde "No Status" a "Todo" usando GraphQL API
12. [ ] Documentar el PROJECT_ID y comandos útiles en el README o docs del proyecto específico
13. [ ] Commitear los workflows y hacer push
14. [ ] **INSTRUIR AL USUARIO** que debe crear el PAT y agregarlo como secret

### G. Comandos de Consulta Útiles

```bash
# Listar todos los proyectos del owner
gh project list --owner [USERNAME] --format json

# Ver detalles de un proyecto
gh project view [PROJECT_NUMBER] --owner [USERNAME] --format json

# Listar items del proyecto
gh project item-list [PROJECT_NUMBER] --owner [USERNAME] --format json

# Listar campos del proyecto
gh project field-list [PROJECT_NUMBER] --owner [USERNAME] --format json
```

### H. Notas Importantes

- **Épicas:** NO añadir al proyecto Kanban. Solo crear la issue con label `Epic` y mantenerla fuera del board.
- **Sub-issues:** Idealmente las tareas deberían crearse como sub-issues de una Épica, pero crear issues regulares vinculadas también funciona.
- **Status "No Status":** Issues nuevas aparecen en "No Status" hasta que el workflow las mueva. Asegurar que el workflow se ejecuta correctamente.
- **Permisos:** El `GITHUB_TOKEN` por defecto tiene permisos suficientes para mover items en el proyecto. Si se necesita un PAT, usar `secrets.PROJECT_PAT`.