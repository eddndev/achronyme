# 🔒 Proteger la Rama `main` - Instrucciones

## ⚠️ IMPORTANTE: Hacer ANTES de trabajar en equipo

### 🌐 Configurar desde GitHub Web (Recomendado)

1. **Ir a:** `https://github.com/[OWNER]/[REPO]/settings/branches`

2. **Click:** "Add branch protection rule"

3. **Branch name pattern:** `main`

4. **Configurar estas reglas:**

#### ✅ Protect matching branches

```
☑️ Require a pull request before merging
   ☑️ Require approvals: 1
   ☑️ Dismiss stale pull request approvals when new commits are pushed
   ☐ Require review from Code Owners (opcional, si creas .github/CODEOWNERS)

☑️ Require status checks to pass before merging
   ☑️ Require branches to be up to date before merging
   Status checks: (Seleccionar cuando estén disponibles)
   - ☑️ Claude Code Review
   - ☑️ Project Board Automation

☑️ Require conversation resolution before merging

☑️ Include administrators
   (Para que TÚ también sigas las reglas - recomendado)

☐ Allow force pushes (NUNCA activar)
☐ Allow deletions (NUNCA activar)
```

5. **Click:** "Create" o "Save changes"

---

## ✅ Verificar que Funciona

Intenta hacer push directo a main:

```bash
# Esto debería FALLAR ❌
git checkout main
echo "test" >> test.txt
git add test.txt
git commit -m "test"
git push origin main

# Deberías ver error:
# remote: error: GH006: Protected branch update failed
```

Si funciona correctamente, **no podrás hacer push directo a main** ✅

---

## 🔄 Ahora el Flujo OBLIGA a usar PRs

```bash
# 1. Crear rama
git checkout -b feature/test

# 2. Hacer cambios y commit
git add .
git commit -m "feat: test"

# 3. Push de la rama
git push origin feature/test

# 4. Crear PR (CLI o web)
gh pr create

# 5. Esperar aprobación
# 6. Mergear desde GitHub web o CLI
gh pr merge [NUM] --squash
```

---

## 📋 Checklist de Seguridad

- [ ] Rama `main` protegida
- [ ] Require PR antes de merge: ✅
- [ ] Require 1 aprobación: ✅
- [ ] Include administrators: ✅ (recomendado)
- [ ] Force push disabled: ✅
- [ ] Deletions disabled: ✅
- [ ] Conversation resolution required: ✅

---

## 🚨 Bypass de Emergencia (Solo si es CRÍTICO)

Si necesitas hacer un hotfix urgente y no puedes esperar review:

1. Ir a: `https://github.com/[OWNER]/[REPO]/settings/branches`
2. Desactivar temporalmente "Include administrators"
3. Hacer el push de emergencia
4. **REACTIVAR inmediatamente** "Include administrators"

**⚠️ Solo usar en emergencias reales.**

---

**Siguiente paso:** Leer `docs/workflow/01-team-workflow.md` para el flujo completo de trabajo en equipo.
