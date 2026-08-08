

# fn research

Cada herramienta de investigación profunda funciona de la misma manera: escribes un tema, busca y obtienes 30 páginas de texto vagamente relevante. Nadie te preguntó lo que realmente querías saber.

Esta herramienta crea un brief de investigación *contigo* antes de buscar cualquier cosa.

## Cómo funciona el brief

Escribes `rs <topic>` y el agente construye un brief estructurado en dos fases — temas raíz y luego subtemas — con tu participación continua.

**Fase 1** — el agente genera 3 temas raíz a partir de tu solicitud. Cada uno es un ángulo de investigación específico, no un encabezado genérico. Cada uno viene con tres propiedades — profundidad, novedad, aplicado — en una escala de 1 a 5. Estas no son solo etiquetas: puedes cambiarlas. Di "haz el primero más profundo" o "menos teórico" y el agente reformulará el tema para que coincida. Lo mismo con los temas en sí: reemplaza, reescribe, intercambia; el brief no se fija hasta que tú digas que sí.

El agente también escribe un resumen de cómo entendió tu intención — "Creo que quieres X por Y, asumo Z sobre tu contexto, dejo W fuera del alcance". Si te entendió mal, díselo y regenerará desde cero.

**Fase 2** — 3 subtemas por cada tema raíz. La misma refinación interactiva. Tú ajustas, el agente reformula. Sin solapamiento, sin relleno genérico.

El brief final es una estructura cerrada de 3×3 que se envía al motor de investigación. Las propiedades y el razonamiento se eliminan: el motor solo ve preguntas de investigación enfocadas y autocontenidas. Pero tú viste el razonamiento completo antes de aprobarlo.

La diferencia: en lugar de buscar "computación cuántica" y esperar lo mejor, estás enviando un brief que indica exactamente qué 9 ángulos investigar y a qué profundidad.

```
rs parallel ultra Quantum computing
```

> **Resumen:** Lo interpreto como curiosidad por dónde está realmente la computación cuántica: no una introducción de libro de texto, sino una mirada honesta a lo que funciona, lo que aún no y lo que importa para alguien que sigue el campo en serio. Asumo que tienes alfabetización técnica, pero no estás construyendo circuitos cuánticos tú mismo. Dejo fuera la criptografía cuántica y la encriptación postcuántica como un dominio separado, y no cubro el currículo de física (espacios de Hilbert, notación de Dirac) a menos que sirva para un punto práctico.
>
> ---
>
> **1. La brecha de qubits entre las demostraciones de laboratorio y la computación útil: qué puede y qué no puede hacer el hardware actual, y dónde están los verdaderos cuellos de botella**
>    Esta es la pregunta de la verdad terrenal: cuántos qubits realmente funcionan juntos de manera confiable, cómo son las tasas de error hoy en día y por qué los titulares de "1000 qubits" no significan lo que parecen.
>
>    **profundidad:** `4` · **novedad:** `3` · **aplicado:** `3`
>
> ---
>
> **2. La corrección de errores como el problema de ingeniería sin resolver que determina si la computación cuántica escala: enfoques actuales, sobrecarga de recursos y cronogramas realistas**
>    Todos están de acuerdo en que la computación cuántica tolerante a fallos necesita corrección de errores. La parte interesante es lo costosa que es: miles de qubits físicos por cada qubit lógico. Este tema profundiza en si ese costo está disminuyendo lo suficientemente rápido.
>
>    **profundidad:** `5` · **novedad:** `4` · **aplicado:** `2`
>
> ---
>
> **3. Los algoritmos que realmente justificarían construir una computadora cuántica: qué problemas tienen una aceleración cuántica probada y qué industrias la notarían primero**
>    El algoritmo de Shor rompió RSA en teoría, Grover da una aceleración cuadrática para la búsqueda, pero ¿cuál es el portafolio realista de problemas donde la computación cuántica supera a la clásica en la práctica, no solo asintóticamente?
>
>    **profundidad:** `3` · **novedad:** `3` · **aplicado:** `5`

## La salida

Me gusta Hokusai, por lo que los PDFs tienen un estilo de grabado en madera — portadas generadas, colores apagados, motivos de olas. Por dentro es limpio: buena tipografía, tabla de contenido, citas estilizadas. Nada recargado, simplemente no es feo.

Ambos motores investigando "transformación de la investigación académica por la IA":
- [Parallel example](./examples/parallel-ai-academic-research.pdf) — 21 páginas, enfoque estratégico
- [Valyu example](./examples/valyu-ai-academic-research.pdf) — 25 páginas, rico en datos

## Inicio rápido

Esta no es una herramienta CLI. La usas a través de un agente de código con IA:

1. Abre la carpeta del proyecto en Claude Code, Codex, Cursor o Junie
2. Escribe `rs <topic>` — el agente se encarga del resto
3. Responde a las preguntas del agente (idioma, profundidad, ángulos de enfoque)
4. Obtén un PDF en `./output/`

El agente lee `AGENTS.md` para obtener sus instrucciones. Tú te enfocas en qué investigar; el agente se encarga de Docker, las APIs y la generación de archivos.

```
rs parallel ultra Rust ownership model

# El agente:
# - Preguntará sobre el idioma, confirmará el proveedor/procesador
# - Construirá un brief de 3×3 de forma interactiva contigo
# - Iniciará un contenedor Docker
# - Generará un informe PDF en ./output/
```

## División (fork) de investigación

¿Ya tienes una ejecución de investigación y quieres ir más allá? `frk` te permite:

- **Re-brief** — ajustar el brief original y ejecutar de nuevo con diferentes ángulos o profundidad
- **Deep-dive** — elegir una sección específica de los resultados e investigarla más a fondo

Ambos modos te muestran un diff del brief original versus el actualizado antes de iniciarse. El fork crea una nueva sesión; la original permanece intacta.

## Comandos

| Comando | Qué hace |
|---------|---------|
| `rs [provider] [processor] <topic>` | Nueva ejecución de investigación |
| `frk` | Dividir (fork) investigación existente |
| `st` | Listar todas las sesiones con estado y rutas de PDF |
| `pdf <topic>` | Regenerar PDF para una sesión |
| `tst` | Ejecutar pruebas en Docker |

## Proveedores

| | Parallel | Valyu | XAI |
|---|----------|-------|-----|
| **Fuentes** | Internet abierto | Internet abierto + académico y propietario | Web + X/redes sociales |
| **Fortaleza** | Síntesis estratégica | Análisis rico en datos, mejores citas | Señales y discurso social |
| **Ideal para** | Decisiones empresariales, planificación de implementación | Investigación académica, recopilación de evidencias | Cobertura social, temas de tendencia |
| **Procesadores** | pro, ultra, ultra2x, ultra4x, ultra8x | fast, standard, heavy | social, full |
| **Velocidad** | 10–40 min | 30–90 min | 5–20 min |

Usa `all` como proveedor (p. ej., `rs all ultra <topic>`) para ejecutar Parallel y luego Valyu en la misma sesión.

## Configuración

### Requisitos

- Docker
- Un agente de código con IA (Claude Code, Codex, Cursor, Junie)

### Variables de entorno

```bash
export PARALLEL_API_KEY="..."   # Acceso a Parallel AI
export VALYU_API_KEY="..."      # Acceso a Valyu
export XAI_API_KEY="..."        # Acceso a XAI
export GEMINI_API_KEY="..."     # Opcional: generación de imagen de portada
export REPORT_FOR="..."         # Opcional: nombre en la atribución del informe
```

### Dependencias de Python

```bash
uv sync
```

### Pruebas

```bash
# Vía agente:
tst

# Manual:
docker build -t research-test -f Dockerfile.test .
docker run --rm research-test
docker run --rm -v "$PWD/tmp_cache:/app/tmp_cache" -e REPORT_FOR research-test -- --ignored --test-threads=1
```

## Licencia

Apache 2.0
