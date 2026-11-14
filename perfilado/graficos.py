#!/usr/bin/env python3
"""Uso: perfilado/graficos.py [directorio-de-salida]"""

import os
import sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SALIDA_POR_DEFECTO = os.path.join(REPO, "docs", "img")

FUENTE = "system-ui, -apple-system, Segoe UI, sans-serif"
SUPERFICIE = "#fcfcfb"
BORDE = "#e1e0d9"
TINTA = "#0b0b0b"
TINTA_2 = "#52514e"
TINTA_3 = "#898781"
GRILLA = "#e1e0d9"
EJE = "#c3c2b7"
SERIE_GLOBALES = "#2a78d6"
SERIE_LOCALES = "#eb6834"

RADIO_EXTREMO_BARRA = 4
SIN_MEDICION = None

TIEMPOS_POR_IMPLEMENTACION = [
    ("Rust (compilado, -O)", 10),
    ("Python 3.14.7 (CPython)", 214),
    ("rabi VM (release)", 280),
    ("rabi intérprete (release)", 602),
]

TIEMPOS_DE_LA_VM_POR_ETAPA = [
    (["VM inicial", "(sin optimizar)"], 1363),
    (["Tras opt. 1", "globales sin copiar", "el nombre"], 678),
    (["VM final", "(todas las", "optimizaciones)"], 280),
]

INTERPRETE_AL_TERMINAR_LA_VM = 945
INTERPRETE_FINAL = 602

REDUCCION_POR_OPTIMIZACION = [
    ("1. Globales sin copiar el nombre", 50.0, SIN_MEDICION),
    ("2. Hasher rápido para las globales", 21.6, SIN_MEDICION),
    ("3. Copiar los valores una sola vez", 15.0, 33.0),
    ("4. LTO", SIN_MEDICION, 12.1),
]


def escapar(texto):
    return (
        texto.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&quot;")
    )


def texto(x, y, contenido, tamanio=13, color=TINTA_2, ancla="start", peso="400"):
    return (
        f'<text x="{x:.1f}" y="{y:.1f}" font-size="{tamanio}" fill="{color}" '
        f'text-anchor="{ancla}" font-weight="{peso}">{escapar(contenido)}</text>'
    )


def numero(x, y, contenido, tamanio=13, color=TINTA, ancla="start", peso="600"):
    return (
        f'<text x="{x:.1f}" y="{y:.1f}" font-size="{tamanio}" fill="{color}" '
        f'text-anchor="{ancla}" font-weight="{peso}" '
        f'style="font-variant-numeric: tabular-nums">{escapar(contenido)}</text>'
    )


def linea_vertical(x, y_desde, y_hasta, color, grosor=1, guiones=None):
    trazo = f' stroke-dasharray="{guiones}"' if guiones else ""
    return (
        f'<line x1="{x:.1f}" y1="{y_desde:.1f}" x2="{x:.1f}" y2="{y_hasta:.1f}" '
        f'stroke="{color}" stroke-width="{grosor}"{trazo}/>'
    )


def linea_horizontal(y, x_desde, x_hasta, color, grosor=1, guiones=None):
    trazo = f' stroke-dasharray="{guiones}"' if guiones else ""
    return (
        f'<line x1="{x_desde:.1f}" y1="{y:.1f}" x2="{x_hasta:.1f}" y2="{y:.1f}" '
        f'stroke="{color}" stroke-width="{grosor}"{trazo}/>'
    )


def barra_horizontal(x0, y, largo, alto, color):
    if largo <= RADIO_EXTREMO_BARRA:
        return (
            f'<rect x="{x0:.1f}" y="{y:.1f}" width="{max(largo, 1):.1f}" '
            f'height="{alto:.1f}" fill="{color}"/>'
        )
    x1 = x0 + largo
    r = RADIO_EXTREMO_BARRA
    return (
        f'<path d="M {x0:.1f} {y:.1f} H {x1 - r:.1f} '
        f'Q {x1:.1f} {y:.1f} {x1:.1f} {y + r:.1f} '
        f'V {y + alto - r:.1f} '
        f'Q {x1:.1f} {y + alto:.1f} {x1 - r:.1f} {y + alto:.1f} '
        f'H {x0:.1f} Z" fill="{color}"/>'
    )


def barra_vertical(x, y_base, alto_barra, ancho, color):
    if alto_barra <= RADIO_EXTREMO_BARRA:
        return (
            f'<rect x="{x:.1f}" y="{y_base - max(alto_barra, 1):.1f}" '
            f'width="{ancho:.1f}" height="{max(alto_barra, 1):.1f}" fill="{color}"/>'
        )
    y1 = y_base - alto_barra
    r = RADIO_EXTREMO_BARRA
    return (
        f'<path d="M {x:.1f} {y_base:.1f} V {y1 + r:.1f} '
        f'Q {x:.1f} {y1:.1f} {x + r:.1f} {y1:.1f} '
        f'H {x + ancho - r:.1f} '
        f'Q {x + ancho:.1f} {y1:.1f} {x + ancho:.1f} {y1 + r:.1f} '
        f'V {y_base:.1f} Z" fill="{color}"/>'
    )


def titulo_del_grafico(x, contenido):
    return texto(x, 34, contenido, tamanio=17, color=TINTA, peso="600")


def documento(ancho, alto, cuerpo, titulo, descripcion):
    partes = [
        '<?xml version="1.0" encoding="UTF-8"?>',
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{ancho}" height="{alto}" '
        f'viewBox="0 0 {ancho} {alto}" font-family="{FUENTE}" role="img" '
        f'aria-labelledby="titulo desc">',
        f'<title id="titulo">{escapar(titulo)}</title>',
        f'<desc id="desc">{escapar(descripcion)}</desc>',
        f'<rect width="{ancho}" height="{alto}" fill="{SUPERFICIE}"/>',
        f'<rect x="0.5" y="0.5" width="{ancho - 1}" height="{alto - 1}" '
        f'fill="none" stroke="{BORDE}"/>',
    ]
    partes.extend(cuerpo)
    partes.append("</svg>")
    return "\n".join(partes) + "\n"


def grafico_implementaciones():
    ancho, alto = 840, 278
    izq, der, arriba, abajo = 215, 95, 70, 36
    plot_ancho = ancho - izq - der
    plot_alto = alto - arriba - abajo
    maximo = 650
    escala = plot_ancho / maximo
    fila = plot_alto / len(TIEMPOS_POR_IMPLEMENTACION)
    grosor = 26

    cuerpo = [titulo_del_grafico(24, "Tiempo del benchmark por implementación")]

    for valor in range(0, maximo + 1, 100):
        x = izq + valor * escala
        cuerpo.append(linea_vertical(x, arriba, arriba + plot_alto, GRILLA))
        cuerpo.append(
            numero(
                x,
                arriba + plot_alto + 20,
                str(valor),
                tamanio=11,
                color=TINTA_3,
                ancla="middle",
                peso="400",
            )
        )

    cuerpo.append(linea_vertical(izq, arriba, arriba + plot_alto, EJE))

    for i, (etiqueta, valor) in enumerate(TIEMPOS_POR_IMPLEMENTACION):
        y = arriba + i * fila + (fila - grosor) / 2
        largo = valor * escala
        cuerpo.append(barra_horizontal(izq, y, largo, grosor, SERIE_GLOBALES))
        cuerpo.append(
            texto(izq - 14, y + grosor / 2 + 4.5, etiqueta, tamanio=13, color=TINTA, ancla="end")
        )
        cuerpo.append(numero(izq + largo + 10, y + grosor / 2 + 4.5, f"{valor} ms"))

    return documento(
        ancho,
        alto,
        cuerpo,
        "Tiempo del benchmark por implementación",
        "Barras horizontales en milisegundos: Rust 10 ms, Python 3.14.7 214 ms, "
        "rabi VM 280 ms, rabi intérprete 602 ms.",
    )


def grafico_evolucion():
    ancho, alto = 840, 388
    izq, der, arriba, abajo = 72, 196, 70, 70
    plot_ancho = ancho - izq - der
    plot_alto = alto - arriba - abajo
    maximo = 1400
    base = arriba + plot_alto
    escala = plot_alto / maximo
    ranura = plot_ancho / len(TIEMPOS_DE_LA_VM_POR_ETAPA)
    grosor = 92

    cuerpo = [titulo_del_grafico(24, "Evolución de la VM a lo largo del desarrollo")]

    for valor in range(0, maximo + 1, 200):
        y = base - valor * escala
        cuerpo.append(linea_horizontal(y, izq, izq + plot_ancho, GRILLA))
        cuerpo.append(
            numero(izq - 12, y + 4, str(valor), tamanio=11, color=TINTA_3, ancla="end", peso="400")
        )

    cuerpo.append(linea_horizontal(base, izq, izq + plot_ancho, EJE))

    referencias = [
        (INTERPRETE_AL_TERMINAR_LA_VM, "intérprete en ese momento", TINTA_3, 1.5, "2 4"),
        (INTERPRETE_FINAL, "intérprete final (make bench)", TINTA_2, 2, "7 5"),
    ]
    for valor, etiqueta, color, grosor_linea, guiones in referencias:
        y = base - valor * escala
        cuerpo.append(linea_horizontal(y, izq, izq + plot_ancho, color, grosor_linea, guiones))
        cuerpo.append(texto(izq + plot_ancho + 10, y - 2, etiqueta, tamanio=11, color=color))
        cuerpo.append(
            numero(izq + plot_ancho + 10, y + 14, f"{valor} ms", tamanio=11, color=color, peso="600")
        )

    for i, (lineas, valor) in enumerate(TIEMPOS_DE_LA_VM_POR_ETAPA):
        x = izq + i * ranura + (ranura - grosor) / 2
        alto_barra = valor * escala
        cuerpo.append(barra_vertical(x, base, alto_barra, grosor, SERIE_GLOBALES))
        cuerpo.append(
            numero(x + grosor / 2, base - alto_barra - 10, f"{valor} ms", tamanio=14, ancla="middle")
        )
        for j, linea in enumerate(lineas):
            cuerpo.append(
                texto(
                    x + grosor / 2,
                    base + 24 + j * 16,
                    linea,
                    tamanio=12,
                    color=TINTA if j == 0 else TINTA_2,
                    ancla="middle",
                    peso="600" if j == 0 else "400",
                )
            )

    cuerpo.append(texto(izq - 60, arriba - 14, "ms", tamanio=11, color=TINTA_3))

    return documento(
        ancho,
        alto,
        cuerpo,
        "Evolución de la VM a lo largo del desarrollo",
        "La VM arranca en 1363 ms, por encima del intérprete, baja a 678 ms tras la primera "
        "optimización y termina en 280 ms, menos de la mitad de los 602 ms del intérprete.",
    )


def grafico_optimizaciones():
    ancho, alto = 870, 308
    izq, der, arriba, abajo = 262, 96, 96, 36
    plot_ancho = ancho - izq - der
    maximo = 55
    escala = plot_ancho / maximo
    grupo = 44
    grosor = 16
    plot_alto = grupo * len(REDUCCION_POR_OPTIMIZACION)

    cuerpo = [titulo_del_grafico(24, "Impacto de cada optimización de la VM")]

    x_leyenda = 24
    for color, etiqueta in (
        (SERIE_GLOBALES, "benchmark con variables globales"),
        (SERIE_LOCALES, "benchmark con variables locales"),
    ):
        cuerpo.append(f'<rect x="{x_leyenda}" y="56" width="11" height="11" rx="2" fill="{color}"/>')
        cuerpo.append(texto(x_leyenda + 17, 66, etiqueta, tamanio=12, color=TINTA_2))
        x_leyenda += 24 + 7.2 * len(etiqueta)

    for valor in range(0, maximo + 1, 10):
        x = izq + valor * escala
        cuerpo.append(linea_vertical(x, arriba, arriba + plot_alto, GRILLA))
        cuerpo.append(
            numero(
                x,
                arriba + plot_alto + 20,
                f"{valor} %",
                tamanio=11,
                color=TINTA_3,
                ancla="middle",
                peso="400",
            )
        )

    cuerpo.append(linea_vertical(izq, arriba, arriba + plot_alto, EJE))

    for i, (etiqueta, globales, locales) in enumerate(REDUCCION_POR_OPTIMIZACION):
        centro = arriba + i * grupo + grupo / 2
        cuerpo.append(texto(izq - 14, centro + 4.5, etiqueta, tamanio=13, color=TINTA, ancla="end"))
        y_barra = centro - grosor - 1
        for valor, color in ((globales, SERIE_GLOBALES), (locales, SERIE_LOCALES)):
            if valor is not SIN_MEDICION:
                largo = valor * escala
                cuerpo.append(barra_horizontal(izq, y_barra, largo, grosor, color))
                es_entero = valor == int(valor)
                decimal = f"{valor:g}" if es_entero else f"{valor:.1f}".replace(".", ",")
                cuerpo.append(
                    numero(izq + largo + 9, y_barra + grosor / 2 + 4.5, f"−{decimal} %", tamanio=12)
                )
            y_barra += grosor + 2

    return documento(
        ancho,
        alto,
        cuerpo,
        "Impacto de cada optimización de la VM",
        "Reducción del tiempo de ejecución por optimización: globales sin copiar el nombre 50 %, "
        "hasher rápido para las globales 21,6 %, copiar los valores una sola vez 15 % con "
        "globales y 33 % con locales, LTO 12,1 % con locales.",
    )


GRAFICOS = [
    ("benchmark-implementaciones.svg", grafico_implementaciones),
    ("evolucion-vm.svg", grafico_evolucion),
    ("impacto-optimizaciones.svg", grafico_optimizaciones),
]


def main():
    salida = sys.argv[1] if len(sys.argv) > 1 else SALIDA_POR_DEFECTO
    os.makedirs(salida, exist_ok=True)
    for nombre, generar in GRAFICOS:
        ruta = os.path.join(salida, nombre)
        with open(ruta, "w", encoding="utf-8") as archivo:
            archivo.write(generar())
        print(os.path.relpath(ruta, REPO))


if __name__ == "__main__":
    main()
