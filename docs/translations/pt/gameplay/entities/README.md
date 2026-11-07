---
source: 9167db67d391d56c5d5d5ba31e5f260c3c7d0f58
---

# Referência de entidades { #entity-reference }

As entidades do pacote de entidades de gameplay, agrupadas pelo que você usaria cada uma. Cada entrada diz para que serve a entidade e
depois lista seus inputs (o que outras entidades podem mandar ela fazer), seus outputs (eventos que você pode ligar adiante) e suas
keys (as configurações no Inspetor). Como os outputs se conectam aos inputs está explicado em [Como funciona o I/O de entidades](../io.md).

| Página | O que você encontra nela |
| --- | --- |
| [Lógica](logic.md) | Ajudantes invisíveis que decidem quando as coisas acontecem: relays, verificações "somente se", portas, flip flops, sorteios e casos, contadores, timers, um evento de início do mapa e linhas do tempo de cutscene |
| [Matemática e valores](math.md) | Ajudantes invisíveis que guardam, calculam e comparam números, como uma pontuação que abre uma porta |
| [Scripts e chamadas](scripting.md) | Executar um trecho de GDScript, chamar o código do seu jogo, tocar uma animação ou imprimir o que uma cadeia está fazendo |
| [Portas, movers e botões](movers.md) | Entidades brush que se movem: portas deslizantes e giratórias, elevadores, trens e botões |
| [Triggers](triggers.md) | Volumes invisíveis que reagem quando um corpo entra neles, para iniciar eventos, ferir, teletransportar, empurrar ou gerar |
| [Spawners e caminhos](actors.md) | Gerar inimigos ou itens, destinos de teletransporte, cantos de caminho e personagens de cutscene com script |
| [Props, luzes e efeitos](effects.md) | Props de física que quebram, modelos estáticos, explosões, texto na tela, luzes que ligam e desligam, som, partículas e sondas de luz |
| [Seções em streaming](sections.md) | Marcos e volumes de conexão para jogar vários mapas como um mundo só, sem tela de carregamento |

## Instalando e personalizando { #installing-and-customizing }

O pacote não faz parte do addon em si. Aperte **Instalar entidades de gameplay** na janela *Configurar este projeto*, ou escolha
*Gameplay > Instalar entidades de gameplay*, e as definições e os scripts dele são copiados para `res://trenchdot/entities`. Até lá o
painel Entidades só lista o núcleo: worldspawn, os ajudantes `func_`, `light`, `light_spot`, as sondas de luz e de reflexo e
`info_player_start`.

As cópias pertencem ao seu projeto. Mude o script de uma entidade em `scripts/` ou as keys e os padrões dela em `definitions/`,
apague as entidades que você não usa ou adicione as suas. Uma atualização do addon oferece um pacote mais novo, mas nunca
sobrescreve um arquivo que você editou. [Configuração do projeto](../../godot/project-setup.md#core-entities-and-the-gameplay-entities-pack) explica as atualizações e
o que acontece quando duas definições têm o mesmo classname.

## Lendo as entradas { #reading-the-entries }

* Argumentos entre parênteses são o que um input aceita ou o que um output repassa, por exemplo `trigger(activator)`
  recebe o corpo ou a entidade que iniciou a cadeia.
* Toda entidade com inputs ou outputs tem uma key `targetname`, exceto `logic_auto`. É o nome que as outras entidades usam para
  chegar até ela.
* Uma key que você não define usa o padrão listado.
* As distâncias são em unidades do mapa, 32 por metro. As velocidades são em metros por segundo, e `omni_range`, `spot_range` e
  `max_distance` são em metros.
* As listas de inputs e outputs são o que os seletores do editor oferecem. Em tempo de execução qualquer método do script da
  entidade funciona como input, então o seletor não é a lista completa.

Para adicionar suas próprias classes, veja [Entidades próprias em GDScript](../custom-entities.md).

## Fora do pacote { #not-in-the-pack }

**O núcleo** vem com o addon e não precisa do pacote. O worldspawn guarda as configurações de sol, luz ambiente, céu, névoa e
streaming, e o Inspetor o mostra quando nada está selecionado. As classes `func_geo`, `func_detail`,
`func_detail_illusionary` e `func_illusionary` do FuncGodot constroem geometria simples. `info_player_start` marca onde o jogador
nasce mas não gera nada por si, o código do seu jogo põe o jogador lá. [`light` e `light_spot`](effects.md) são as
luzes que ligam e desligam.

Entidades brush e entidades de modelo colidem por padrão, veja [Colisão](collision.md) para as keys que mudam isso.

**O jogador** não é uma entidade, é uma cena do seu próprio jogo. O pacote espera que ele esteja no grupo `player` e na
camada de colisão 1, onde os triggers olham, com um método `take_damage(amount, source)` que os triggers de dano e as explosões
chamam. Um jogador em outra camada precisa dessa camada em *Trigger Collision Mask*, veja
[Triggers](triggers.md#shared-inputs-outputs-and-keys). Veja [`godot/demo/player.gd`](https://github.com/Paraxdev/TrenchDot/blob/main/godot/demo/player.gd) para um
exemplo completo.
