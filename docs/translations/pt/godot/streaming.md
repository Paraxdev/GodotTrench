---
source: 0215c0eddc11970f765e58b4ce44056eebf2815e
---

# Streaming de mapas grandes { #streaming-big-maps }

Mapas externos grandes podem esconder o que está longe, de modo que o mapa custa o que o jogador vê, e não tudo o que ele guarda.
A construção separa as malhas do mapa em chunks numa grade, e só os chunks perto da câmera são desenhados.

## Ligando { #turning-it-on }

1. Não selecione nada

    Aperte Esc para que nada fique selecionado. O Inspetor à direita então mostra *Mapa (worldspawn)*.

2. Adicione `chunk_streaming`

    Digite `chunk_streaming` no campo *chave* lá embaixo, `1` em *valor*, e clique em **Adicionar**. (Se ele já estiver
    na lista, é só ligá-lo ali.)

    ![O Inspetor do worldspawn com chunk_streaming digitado no campo de chave, 1 no campo de valor e o botão Adicionar sob o mouse](../assets/streaming/worldspawn-add-key.png)

    Você deve ver ele na lista junto com as outras chaves do worldspawn:

    ![O Inspetor do worldspawn com chunk_streaming marcado](../assets/streaming/worldspawn-chunk-streaming.png)

3. Salve o mapa

    A próxima construção adiciona um `TrenchDotStreamer` ao mapa.

(Opcionalmente) adicione `chunk_size` e `load_radius` do mesmo jeito:

| Chave | Padrão | Significado |
| --- | --- | --- |
| `chunk_size` | 2048 | Tamanho do chunk em unidades do mapa, 64 m. Chunks menores escondem com mais precisão, mas custam mais para verificar |
| `load_radius` | 8192 | A que distância da câmera os chunks continuam visíveis, em unidades do mapa, 256 m |

O streaming só esconde malhas. A colisão e os scripts continuam rodando em todo lugar, então ele economiza custo de renderização, não de física nem
de lógica. Overlays, conteúdo criado em tempo de execução e qualquer coisa numa porta ou trem em movimento nunca são escondidos.

O streamer segue a câmera atual da viewport, ou um `info_player_start` enquanto não há câmera. Para prendê-lo a
uma câmera, defina o `camera_path` dele por código depois da construção, já que uma reconstrução substitui o streamer. Ele só roda no
jogo, então o editor do Godot sempre mostra o mapa inteiro.

Conforme chunks entram e saem, o streamer emite `area_loaded(key, bounds)` e `area_unloaded(key, bounds)`, para você poder
fazer streaming do seu próprio conteúdo junto:

```gdscript
streamer.area_loaded.connect(func(key, bounds): spawn_wildlife(bounds))
```

## Streaming de seções { #streaming-sections }

Quer jogar vários mapas como um mundo só, sem telas de carregamento? Divida-o em seções e junte-as com salas conectoras.
Só uma seção fica na árvore de cena por vez. A próxima carrega em segundo plano enquanto o jogador
explora, e entra no lugar enquanto o jogador atravessa um conector, alinhada para que a sala ao redor dele continue exatamente
igual. Isso precisa do [pacote de entidades de gameplay](../gameplay/entities/README.md#installing-and-customizing).

1. Construa uma sala conectora

    Um corredor com uma curva de cada lado, para que nenhuma ponta seja visível do meio. Ponha um `info_landmark` no
    meio, e dois volumes `func_section_stream`, um cobrindo cada metade, se encontrando no meio. Adicione-a à
    [biblioteca de prefabs](../editor/prefab-library.md) para que toda cópia seja igual.

2. Cole onde dois mapas se encontram

    Em cada mapa que encontra outro, cole o conector onde os dois se encontram e dê nome ao seu landmark, por exemplo
    `lm_hall_cellar`. Defina o `section` de cada metade como a seção daquele lado e `landmark` como o nome do landmark.
    Os dois mapas recebem a mesma cópia com os mesmos nomes, virada do mesmo jeito.

3. Pré-calcule a iluminação em todo mapa de seção

    O SDFGI só percebe a geometria que uma troca traz quando as cascatas dele rolam, então uma seção que entrou na troca ficaria sem luz.
    A iluminação pré-calculada vem junto com a seção.

4. Faça uma cena por seção

    Uma cena por seção, com o mapa construído dela, do jeito de sempre.

5. Adicione o streamer

    Adicione um nó `TDSectionStreamer` à cena do seu mundo e defina o `first_section` dele e o jogador dele, veja
    [TDSectionStreamer](../gameplay/entities/sections.md#tdsectionstreamer).

> [!TIP]
> Mantenha a posição de um landmark num múltiplo de 480 unidades do mapa, ou de qualquer tamanho em que todas as texturas do conector
> se repitam, para que as texturas se alinhem do mesmo jeito nas duas cópias.
