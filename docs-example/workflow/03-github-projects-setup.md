# 🔧 Setup de GitHub Projects - Guía de Configuración

**Esta guía es genérica y aplica a cualquier repositorio que use la metodología de AGENTS.md**

## ⚠️ IMPORTANTE: Crear Personal Access Token (PAT)

El workflow de automatización **requiere un PAT** porque `GITHUB_TOKEN` no tiene permisos suficientes para Projects.

### Paso 1: Crear el PAT

1. **Ir a:** https://github.com/settings/tokens
2. **Click:** "Generate new token" → "Generate new token (classic)"
3. **Nombre:** `PROJECT_AUTOMATION`
4. **Seleccionar scopes:**
   - ✅ `repo` (Full control of private repositories)
   - ✅ `project` (Full control of projects)
5. **Click:** "Generate token"
6. **⚠️ COPIAR EL TOKEN** (solo se muestra una vez)

### Paso 2: Agregar como Secret

1. **Ir a:** `https://github.com/[OWNER]/[REPO]/settings/secrets/actions`
2. **Click:** "New repository secret"
3. **Name:** `PROJECT_PAT`
4. **Secret:** Pegar el token copiado
5. **Click:** "Add secret"

---

## ✅ Verificar que Funciona

Una vez agregado el secret `PROJECT_PAT`:

1. **Asignarte una issue:**
   ```bash
   gh issue edit [NUM] --add-assignee "@me"
   ```

2. **Verificar workflow:**
   ```bash
   gh run list --limit 3
   ```
   Debería decir `success` en lugar de `failure`

3. **Verificar en el proyecto:**
   - La issue debería moverse automáticamente a "In Progress"
   - URL: `https://github.com/users/[OWNER]/projects/[NUM]`

---

## 🤖 Automatizaciones Activas

Una vez configurado el PAT, estas automatizaciones funcionan:

- ✅ **Nueva issue → Todo** (al crear issue)
- ✅ **Asignar → In Progress** (al asignarte)
- ✅ **Añadir label "Status: In Progress" → In Progress**
- ✅ **Cerrar issue → Done**
- ✅ **Auto-añadir al proyecto** si la issue tiene label `Sprint: X`

---

## 📋 Información del Proyecto

- **Proyecto:** `https://github.com/users/[OWNER]/projects/[NUM]`
- **PROJECT_ID:** (Ver en output de `gh project create`)
- **Workflow:** `.github/workflows/project-board-automation.yml`

---

## 🚨 Troubleshooting

### Error: "Resource not accessible by integration"
**Causa:** Falta el secret `PROJECT_PAT`
**Solución:** Seguir los pasos arriba para crear y agregar el PAT

### La issue no se movió automáticamente
1. Verificar que el workflow se ejecutó: `gh run list --limit 5`
2. Ver logs si falló: `gh run view [RUN_ID] --log`
3. Verificar que la issue NO tiene label `Epic` (las épicas no se mueven)

### Necesito mover issues manualmente
```bash
# Ver script en AGENTS.md sección 5.D
```

---

**Documentación completa:** Ver `docs/AGENTS.md` sección 5
