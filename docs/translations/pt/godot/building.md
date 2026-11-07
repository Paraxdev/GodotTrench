---
source: 755d17f52aebf0b32513b6e22a04abb368badeab
---

# Construindo mapas { #building-maps }

O TrenchDot salva um arquivo `.tdm` e nunca escreve cenas do Godot por conta própria. No Godot, o nó `FuncGodotMap` do addon lê esse arquivo e gera as malhas, a colisão e os nós de entidade. O resultado é salvo junto com a sua cena como qualquer nó feito à mão, então nada é reconstruído em tempo de execução a menos que você peça.

## O nó FuncGodotMap { #the-funcgodotmap-node }

1. Adicione o nó

    No editor do Godot, aperte Ctrl+A na sua cena, procure `FuncGodotMap` e adicione.

2. Aponte para o seu mapa

    Com o nó selecionado, defina *Local Map File* no Inspetor do Godot como o `.tdm` que você salvou.

3. Construa

    Clique em **Build Map** no topo do Inspetor. Você deve ver o mapa na viewport do Godot, e o nó ganha filhos para as malhas e as entidades. Salve a cena com Ctrl+S para mantê-los. **Clear Map** os remove de novo.

Estas são as configurações que você vê no Inspetor:

| Propriedade | O que faz |
| --- | --- |
| Local Map File | O caminho `res://` do `.tdm` a construir. Esta é a escolha normal. |
| Global Map File | Um arquivo de mapa fora do projeto. Quando os dois estão definidos, este vence. |
| Auto Rebuild On Save | Reconstrói o nó sempre que o TrenchDot salva este mapa, enquanto a cena dele é a aba atual no editor do Godot, veja [Reconstruir ao salvar](live.md#rebuild-on-save). Ligado por padrão. |
| Map Settings | De onde vêm as texturas, os materiais e as definições de entidade e qual o tamanho de uma unidade do mapa. Deixado vazio, usa a configuração de projeto `func_godot/default_map_settings`, veja [Configuração do projeto](project-setup.md). Um nó de mapa adicionado com um addon mais antigo guardava aqui as configurações de mapa padrão do addon, e a construção dele avisa disso no Output do Godot, limpe o campo para seguir a configuração do projeto. |
| Use Baked Lighting | Ilumina o mapa com a iluminação [pré-calculada no editor](baked-lighting.md). Ligado por padrão, desmarque para usar outro lightmapper. |
| Warm Up Shaders | Compila os shaders do mapa enquanto ele carrega, para que os primeiros segundos de jogo não engasguem. Veja [Aquecimento de shaders](warm-up.md). |

O addon só constrói arquivos `.tdm`. Para usar um `.map` do Quake ou um `.vmf` do Hammer, abra-o com *Arquivo > Importar* no TrenchDot e salve-o como `.tdm` primeiro.

> [!WARNING]
> Uma construção apaga todos os filhos do nó do mapa, exceto os [overlays](overlays.md). Mantenha seus próprios nós ao lado do nó do mapa, ou em um overlay.

## Três jeitos de construir { #three-ways-to-build }

| Gatilho | O que é construído | Quando usar |
| --- | --- | --- |
| Salvar no TrenchDot | Todo `FuncGodotMap` que usa esse arquivo na cena aberta no editor do Godot | O ciclo do dia a dia, salvar e olhar o resultado no Godot |
| *Godot > Construir no Godot* | O mapa como ele está no TrenchDot agora, salvo ou não, sem escrever o arquivo | Para testar no Godot uma alteração não salva sem se comprometer com ela |
| **Build Map** no Godot | O arquivo salvo | Quando o TrenchDot está fechado, ou para reconstruir uma cena que você acabou de abrir |

Os dois primeiros precisam de um editor do Godot com este projeto aberto, e só mexem na cena aberta lá. Uma reconstrução marca essa cena como modificada, então salve-a no Godot para manter o resultado.

Enquanto o Godot não tem o projeto aberto, *Construir no Godot* fica acinzentado. Ele também precisa que o mapa tenha sido salvo no projeto uma vez, para o Godot saber qual `FuncGodotMap` o usa.

![O menu Godot com Construir no Godot sob o mouse](../assets/building/menu-build-in-godot.png)

## Construindo por código { #building-from-code }

Você também pode construir um mapa enquanto o jogo roda, por exemplo para carregar uma fase escolhida num menu:

```gdscript
var map := FuncGodotMap.new()
map.map_settings = load("res://demo/demo_map_settings.tres")
map.local_map_file = "res://demo/maps/scripted_scene.tdm"
add_child(map)
map.build()
```

O nó emite `build_complete` quando termina, ou `build_failed` quando o arquivo do mapa não existe ou não é um `.tdm`.

## Ambiente e sol { #environment-and-sun }

As chaves de céu, névoa e sol do worldspawn constroem um `WorldEnvironment` e uma luz chamada `sun` dentro do mapa, então um mapa aparece iluminado sem nenhuma configuração no Godot. Aperte Escape para que nada fique selecionado, o Inspetor então mostra *Mapa (worldspawn)* com essas chaves listadas. A chave `environment` escolhe o que é construído. Uma chave que você deixa sem definir fica fora do mapa, então um mapa sem nenhuma delas não constrói ambiente nenhum. Se um projeto feito com um addon mais antigo não lista as chaves, atualize o addon e [exporte a configuração do jogo](project-setup.md#the-game-config) de novo.

| `environment` | Constrói |
| --- | --- |
| `1` | O ambiente e o sol. Este é o padrão. |
| `sun_only` | Só o sol, para um jogo que traz o próprio `WorldEnvironment` |
| `0` ou `none` | Nada |

Se a cena já tem um `WorldEnvironment` ou um `DirectionalLight3D` próprio, o mapa o mantém e não acrescenta nenhum, então um jogo com visual próprio sempre vence. Quando vários mapas compartilham uma cena, o primeiro construído a ilumina.

## Relatório de construção { #build-report }

O `build()` retorna um Dictionary, também guardado em `build_report`. Ele diz quanto tempo cada etapa levou, quantas entidades de cada classe foram construídas e quantos vértices o mapa tem. O mais útil é a lista do que a construção não encontrou, que costuma ser um erro de digitação ou um arquivo que foi renomeado ou nunca entrou no commit:

| Chave | O que lista | O que você vê no jogo |
| --- | --- | --- |
| `missing_classes` | Classes de entidade que nenhuma definição de entidade descreve | Uma entidade brush mantém a malha, mas fica sem script e sem colisão, uma entidade ponto é um `Marker3D` vazio, veja [Entidades que não fazem nada](entities-troubleshooting.md) |
| `missing_models` | Caminhos de modelo que não existem, com as entidades que usam cada um | O prop fica sem modelo |
| `missing_materials` | Texturas de face sem arquivo de material nem imagem | As faces mostram a textura provisória |
| `unknown_tool_textures` | Texturas de ferramenta como `special/clip` que as configurações do mapa nomeiam de outro jeito | As faces continuam sem desenhar nada, mas um brush de origin não define a origem da entidade dele |
| `unresolved_targets` | Saídas e chaves de alvo cujo alvo não bate com nenhum targetname do mapa ou dos overlays | A saída dispara no vazio |

Para imprimi-lo depois de cada construção, marque *Print Build Report* em *Build Flags* no nó, ou adicione `FuncGodotMap.BuildFlags.PRINT_REPORT` a `build_flags` no código. Um teste ou uma checagem de CI pode ler o Dictionary no lugar:

```gdscript
var report := map.build()
if not report["missing_models"].is_empty():
	push_error("missing models: %s" % [report["missing_models"]])
```

Toda construção também imprime um aviso no Output do Godot para o que não encontrou: um para todas as classes ausentes, e um para cada modelo, material ou textura de ferramenta ausente, não importa quantos props compartilhem o caminho. Depois de um salvamento, o TrenchDot lista os mesmos avisos no painel Problemas.

> [!NOTE]
> Os overlays que você adiciona depois do `build()` não existem quando o relatório procura alvos, então os targetnames deles aparecem em `unresolved_targets`.

## Mapas danificados { #damaged-maps }

Um mapa danificado ainda é construído com as partes que puderam ser lidas, com um aviso `[TDM]` para cada parte que não pôde. Veja [o formato de mapa .tdm](../format/recovery.md).
