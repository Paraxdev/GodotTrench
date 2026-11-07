---
source: 63266bb349fb8b02f543d43498bfdee3d8c3f3cd
---

# Portas, movers e botões { #doors-movers-and-buttons }

> [!NOTE]
> Estas entidades vêm com o pacote de entidades de gameplay, veja
> [Instalando e personalizando](README.md#installing-and-customizing).

Entidades brush que se movem como um corpo só e levam tudo que está em cima delas, então um elevador leva o jogador junto.

> [!NOTE]
> Portas e botões não reagem ao jogador por conta própria. O script do seu jogador chama `use(activator)` neles,
> por exemplo a partir de um raio de interação, veja [Botão abre uma porta](../../tutorials/button-door.md). Defina `interact`
> como 0 para um que só o I/O ou um script deva mover.

## func_door { #func_door }

Uma porta deslizante, escotilha ou veneziana. Ela se move por `travel` ao abrir e volta ao fechar, arraste a alça na
viewport para definir. Uma porta trancada ignora `open` e `use` e dispara `locked_use` no lugar, para uma mensagem de "trancada".

| Key | Padrão | O que faz |
| --- | --- | --- |
| `travel` | 0 64 0 | Deslocamento pelo qual ela desliza, em unidades do mapa |
| `speed` | 2.0 | Metros por segundo |
| `wait` | -1 | -1 fica aberta, um valor positivo a fecha de novo depois desse tanto de segundos |
| `locked` | 0 | Começa trancada |
| `start_open` | 0 | Começa na posição aberta |
| `interact` | 1 | Abre quando o jogador chama `use` nela |

* **Inputs:** `open`, `close`, `toggle`, `lock`, `unlock`, `use(activator)`
* **Outputs:** `opened`, `closed`, `started_opening`, `started_closing`, `locked_use(activator)`

## func_door_rotating { #func_door_rotating }

Uma porta ou portão que gira em torno de uma dobradiça que você arrasta para o lugar na viewport. `open` recebe o ativador para que
`open_away` possa girá-la para longe dele em vez de na cara dele.

| Key | Padrão | O que faz |
| --- | --- | --- |
| `hinge` | 0 0 0 | Posição da dobradiça em relação ao centro da porta |
| `axis` | y | Eixo de giro. `y` é uma dobradiça em pé, `x` ou `z` fazem uma escotilha ou ponte levadiça |
| `open_angle` | 95 | Graus de giro, negativo gira para o outro lado |
| `speed` | 120 | Graus por segundo |
| `open_away` | 1 | Gira para longe de quem a abre |
| `wait`, `locked`, `start_open`, `interact` | -1, 0, 0, 1 | Como em `func_door` |

* **Inputs:** `open(activator)`, `close`, `toggle(activator)`, `lock`, `unlock`, `use(activator)`
* **Outputs:** os mesmos de `func_door`

`func_gate` é um alias antigo com só `hinge`, `open_angle` 100 e `speed` 90 como keys. Use `func_door_rotating`.

## func_platform { #func_platform }

Um elevador ou plataforma móvel entre o ponto inicial e `start + travel`, para elevadores ou pisos que cruzam um fosso.

* **Inputs:** `start`, `stop`, `toggle`, `go_to_end`, `go_to_start`
* **Outputs:** `reached_end`, `reached_start`, `started`
* **Keys:** `travel` 0 128 0, `speed` 2.0, `wait` 1.0, `mode` 0, `start_active` 0 (se move desde o carregamento do mapa)

| `mode` | O que `start` faz | Uso típico |
| --- | --- | --- |
| 0, toggle | Move para a outra ponta | Um elevador chamado por um botão |
| 1, ping pong | Vai e volta até `stop`, esperando `wait` segundos em cada ponta | Uma plataforma que não para de circular |
| 2, once | Move até o fim e fica lá | Uma ponte que se estende uma vez |

## func_train { #func_train }

Segue uma cadeia de entidades `path_corner` a partir de `target`, para qualquer coisa que percorre uma rota com várias paradas, como um bonde
ou um carrinho sobre trilhos. O `wait` e o `speed` de cada canto valem na chegada. Ele começa a se mover no carregamento do mapa e dá voltas, então defina
`start_active` ou `loop` como 0 para uma viagem única.

* **Inputs:** `start`, `stop`, `toggle`
* **Outputs:** `arrived(corner)`, `finished` (só quando não há loop)
* **Keys:** `target`, `speed` 3.0, `loop` 1, `start_active` 1, `orient` 0 (1 o vira para onde ele vai)

## func_button { #func_button }

Afunda por `travel` enquanto apertado e dispara `pressed`. Ele volta a sair depois de `wait` segundos, enquanto -1 o mantém apertado
para um interruptor de uso único. Um botão trancado dispara `locked_use` no lugar, útil para um painel que precisa de energia primeiro.

* **Inputs:** `press(activator)`, `release`, `lock`, `unlock`, `use(activator)`
* **Outputs:** `pressed(activator)`, `released`, `locked_use(activator)`
* **Keys:** `travel` 0 -2 0, `wait` 1.0, `locked` 0, `interact` 1
