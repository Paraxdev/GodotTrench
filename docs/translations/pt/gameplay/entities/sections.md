---
source: 721efa7c5ececa362093af607d2e2c2faa92ad9b
---

# Seções em streaming { #streamed-sections }

> [!NOTE]
> Estas entidades vêm com o pacote de entidades de gameplay, veja
> [Instalando e personalizando](README.md#installing-and-customizing).

Entidades para jogar vários mapas como um mundo só, sem tela de carregamento. Cada mapa é uma seção, e as seções se encontram em uma
sala de conexão da qual os dois mapas guardam uma cópia idêntica. [Seções em streaming](../../godot/streaming.md#streaming-sections)
mostra o passo a passo da configuração.

## info_landmark { #info_landmark }

O ponto compartilhado de uma sala de conexão. Ponha um no meio da sala, com o mesmo nome nas duas cópias. Quando a
próxima seção entra, ela é posicionada de modo que o landmark dela caia exatamente onde estava o antigo.

* **Keys:** `targetname`

## func_section_stream { #func_section_stream }

Uma metade de uma sala de conexão, um volume invisível. Enquanto o jogador está nele, a seção que ele nomeia é a que fica
carregada. Uma conexão precisa de dois, um de cada lado, e a linha onde eles se encontram é onde as seções trocam, então ponha-a onde
nenhuma ponta da sala possa ser vista, por exemplo no meio de um corredor em forma de Z.

* **Keys:** `section` (a seção deste lado: o caminho da cena dela, como `res://levels/pools_2.tscn`, ou um nome encontrado em
  `sections_dir` do streamer), `landmark` (o `targetname` do `info_landmark` da sala)

## TDSectionStreamer { #tdsectionstreamer }

Não é uma entidade, mas o nó que executa o streaming. Adicione-o à cena do seu mundo e defina:

| Propriedade | O que é |
| --- | --- |
| `first_section` | A seção em que o jogo começa, um caminho de cena ou um nome |
| `sections_dir` | Pasta onde os nomes curtos são procurados, como `name.tscn` ou `name.scn` |
| `player_scene` | O jogador a ser gerado no `info_player_start` da primeira seção |
| `player_path` | Ou um jogador que já está na cena. Sem nenhum dos dois, fica o primeiro nó do grupo `player` que uma seção traz |
| `recenter_distance` | Metros que um circuito de seções pode se afastar da origem antes de o mundo inteiro voltar, 8000 |

Ele emite `section_changed(from, to)` depois de cada troca.
