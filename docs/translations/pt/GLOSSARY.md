---
source: 62dc81f5978f85583c7c7cd6083e20459a306923
---

# Glossário { #glossary }

## cursor 3D { #3d-cursor data-forms="cursores 3D" }
O ponto no mundo sob o mouse, na vista em que você passou o mouse por último. Tudo que você posiciona no cursor cai ali, como uma entidade do botão Posicionar no painel Entidades ou um terreno novo.

## ativador { #activator data-forms="ativadores" }
O jogador ou corpo que iniciou uma cadeia de I/O de entidades, em geral quem apertou o botão ou entrou no trigger. Os passos seguintes da cadeia podem ter !activator como alvo e agir exatamente sobre esse.

## albedo { #albedo }
A cor base de um material, a textura simples antes de iluminação, mapas normais ou brilho entrarem em cena. O Godot chama a textura de cor de um material de textura albedo.

## alpha scissor { #alpha-scissor }
Um modo de transparência do Godot que corta a textura de forma seca em um limite de alpha, então cada pixel fica totalmente visível ou totalmente oculto. Serve para grades, folhas e decals de borda dura, e, ao contrário do alpha blending, ainda projeta sombras.

## autoload { #autoload data-forms="autoloads" }
Um script ou cena que o Godot carrega uma vez na inicialização e mantém vivo durante o jogo todo, configurado em Project Settings > Globals > Autoload. O código do jogo inteiro, como um contador de pontos ou o listener de hot reload, mora em um.

## iluminação pré-calculada { #baked-lighting data-forms="iluminações pré-calculadas" }
Luz que é calculada uma vez de antemão e guardada, em vez de ser calculada a cada frame. Godot > Pré-calcular iluminação… grava sol, céu, lâmpadas e materiais que brilham em um lightmap, e o Godot o desenha por um LightmapGI.

## bloom { #bloom }
O brilho suave que lâmpadas fortes e materiais que brilham espalham pelos pixels ao redor. Ele é ligado com a key glow_intensity do worldspawn.

## booleano { #boolean data-forms="booleanos|booleana|booleanas" }
Um valor que é verdadeiro ou falso, como se a energia está ligada. As entidades logic_branch, logic_gate e logic_flipflop guardam e combinam booleanos.

## brush { #brush data-forms="brushes" }
Um sólido convexo, como uma caixa ou uma cunha, o bloco básico de construção de um nível. Você desenha brushes nas vistas e os combina com CSG, e o Godot transforma cada um em uma malha com colisão correspondente.

## entidade brush { #brush-entity data-forms="entidades brush" }
Uma entidade que tira sua forma de brushes que você construiu, como uma porta, um botão ou um trigger. Você desenha os brushes primeiro e depois os transforma na entidade, ao contrário de uma entidade ponto, que fica em um único lugar.

## convexa { #convex data-forms="convexo|convexos|convexas" }
Abaulada para fora em todo lugar, sem reentrâncias nem buracos, como uma caixa, uma cunha ou um cilindro. Todo brush precisa ser convexo, por isso o editor recusa movimentos de vértice que o amassariam. Para formas côncavas, converta o brush em malha.

## cordon { #cordon data-forms="cordons" }
Uma caixa que limita o mapa a uma área enquanto você trabalha nele. Com o cordon ativado, os objetos fora da caixa ficam ocultos e são deixados de fora de uma exportação com cordon.

## CSG { #csg }
Geometria sólida construtiva, a construção de formas combinando sólidos, como subtrair um brush de outro para abrir uma porta. Os comandos de CSG do editor são Subtrair, Esvaziar, Fusão convexa e Intersectar.

## decal { #decal data-forms="decals" }
Uma textura aplicada sobre uma superfície, como uma mancha, uma rachadura ou um pôster. No TrenchDot, um decal é uma malha fina que fica sobre a face sem tremular, posicionada ao segurar Alt enquanto você solta um material em uma face. Um decal projetado é, em vez disso, uma entidade que projeta um material em tudo que está dentro da sua caixa.

## desladrilhar { #detile data-forms="desladrilhamento" }
Um ajuste de camada de terreno de 0 a 1 que gira e desloca cada repetição da textura de forma aleatória, para que um campo grande não mostre uma grade óbvia. Ele custa leituras extras de textura, então deixe em 0 onde a repetição não aparece.

## displacement { #displacement data-forms="displacements" }
Uma face de brush de quatro lados dividida em uma grade de vértices que podem ser erguidos ou abaixados, como no Hammer. Ela dá um terreno irregular dentro de um nível construído com brushes. Áreas externas grandes são melhor construídas como um terreno.

## definição de entidade { #entity-definition data-forms="definições de entidade" }
A descrição de uma classe de entidade em um FGD: seu classname, keys, inputs e outputs, e o que o Godot constrói para ela, como a classe do nó e o script. No Godot, é um recurso FuncGodotFGDPointClass ou FuncGodotFGDSolidClass, listado em Entity Definitions de um arquivo FGD.

## atenuação { #falloff data-forms="atenuações" }
Como um pincel, como o da ferramenta Mistura, esmaece do centro até a borda. Uma atenuação suave dá bordas macias, uma constante pinta o disco todo com a mesma força.

## FGD { #fgd }
A definição das classes de entidade de um jogo, com seus nomes, keys, inputs e outputs. O nome vem dos arquivos Forge Game Data do Hammer. No Godot, é um recurso FuncGodotFGDFile, e a configuração do jogo o repassa ao editor.

## fixup { #fixup data-forms="fixups" }
Um prefixo de nome definido em uma instância de prefab. Com um fixup p1, uma entidade chamada door dentro dela vira p1-door, então várias cópias de um prefab mantêm suas ligações separadas.

## func_detail { #func_detail }
Uma entidade brush do FuncGodot para geometria estática com colisão, construída como um nó próprio, separado dos brushes do mundo. Ela não adiciona oclusor, então nunca esconde da renderização os objetos atrás dela.

## func_illusionary { #func_illusionary }
Uma entidade brush do FuncGodot que é visível mas não tem colisão, então os jogadores a atravessam. O tutorial da ilha no mar usa uma para a água.

## FuncGodot { #funcgodot }
O plugin do Godot para construir níveis a partir de arquivos de mapa no estilo Quake. O addon do TrenchDot é um fork dele, por isso o nó se chama FuncGodotMap e seus recursos começam com FuncGodot.

## configuração do jogo { #game-config data-forms="configurações do jogo" }
O arquivo trenchdot_game.json na raiz do projeto Godot, gravado pelo addon. Ele diz ao editor quais entidades, pastas de textura e escala de unidades seu projeto usa, e o editor o recarrega sempre que o addon o grava de novo.

## pacote de entidades de gameplay { #gameplay-entities-pack data-forms="pacotes de entidades de gameplay" }
As portas, botões, elevadores, triggers, lógica, spawners, props e efeitos da referência de entidades. Eles não fazem parte do addon em si. Instalar o pacote copia as definições e os scripts deles para res://trenchdot/entities, onde passam a pertencer ao seu projeto e você pode mudá-los à vontade.

## glTF { #gltf }
Um formato de arquivo aberto para modelos e cenas 3D que o Blender, o Godot e a maioria das ferramentas 3D leem. Um arquivo .glb é a forma binária dele, que guarda a geometria e as texturas em um só arquivo.

## grupo do Godot { #godot-group data-forms="grupos do Godot" }
Uma etiqueta com nome em nós do Godot, definida em Node > Groups ou pelo código, que permite a código e a I/O de entidades achar todos eles de uma vez. Um alvo como @enemies alcança todo nó do grupo enemies.

## Hammer { #hammer }
O editor de níveis da Valve para jogos do motor Source, como Half-Life 2. O TrenchDot toma emprestado o modelo de inputs e outputs de entidades dele e consegue importar seus mapas .vmf.

## hot reload { #hot-reload }
Reconstruir os mapas de um jogo que já está em execução a cada vez que você salva no TrenchDot, para não precisar reiniciá-lo para ver uma mudança. Ele precisa do autoload trenchdot_hot_reload.gd no jogo.

## hotspot { #hotspot data-forms="hotspots" }
Um retângulo marcado em uma trim sheet e guardado no arquivo .hotspots.json da textura. Ajuste ao hotspot encaixa as faces selecionadas no retângulo que melhor corresponde a elas, então uma trim sheet pode texturizar uma sala rapidamente.

## lightmap { #light-map data-forms="lightmaps" }
Uma textura que guarda a luz que chega a cada ponto da geometria estática, rebatidas incluídas. Cada face recebe uma área própria dela.

## sonda de luz { #light-probe data-forms="sondas de luz" }
Um ponto no espaço aberto que guarda a luz que chega de todas as direções. Objetos em movimento pegam sua luz das sondas ao redor, então combinam com a iluminação pré-calculada da geometria estática.

## LightmapGI { #lightmapgi }
O nó do Godot que desenha um lightmap e suas sondas de luz. Um mapa construído a partir de um .tdm com iluminação pré-calculada recebe um chamado baked_lighting.

## prévia iluminada { #lit-preview }
Um modo de sombreamento da vista 3D do editor, alternado com F3, que mostra as luzes do mapa, as sombras do sol, o céu e a névoa. Bom para avaliar a iluminação antes de construir no Godot, mas só desenha as 64 luzes pontuais e spot mais fortes.

## Live Link { #live-link }
A conexão entre o TrenchDot e o editor do Godot, um servidor local que o addon executa na porta 7842. Reconstruir ao salvar, o modo ao vivo e o status do Godot na barra de ferramentas passam todos por ele.

## modo ao vivo { #live-mode }
Envia suas edições para a cena aberta no editor do Godot enquanto você as faz, antes de salvar, para você avaliar iluminação e escala no renderizador do próprio Godot. Ligue com o botão de link na barra de ferramentas ou em Godot > Modo ao vivo.

## porta lógica { #logic-gate data-forms="portas lógicas" }
Uma regra que transforma duas entradas verdadeiras ou falsas em uma resposta, como and (as duas são verdadeiras) ou xor (exatamente uma é). A entidade logic_gate oferece as mais comuns.

## cena principal { #main-scene data-forms="cenas principais" }
A cena que o Godot executa quando você roda o projeto, definida em Project > Project Settings > Application > Run. Executar projeto no TrenchDot e o botão Play no Godot a iniciam.

## configurações do mapa { #map-settings }
Um recurso FuncGodotMapSettings que diz de onde vêm as texturas, os materiais e as definições de entidade e quantas unidades de mapa formam um metro. Um FuncGodotMap pode ter o seu próprio, senão vale o padrão do projeto.

## MCP { #mcp }
Model Context Protocol, uma forma padrão de agentes de IA e scripts chamarem as ferramentas de um programa. O editor pode executar um servidor MCP para que um agente inspecione e edite mapas. Você não precisa dele para construir níveis, e esta documentação esconde as partes de MCP até você ligar a chave MCP no cabeçalho.

## MultiMeshes { #multimeshes }
A forma do Godot de desenhar muitas cópias de uma malha em uma única passada. Árvores, pedras e grama espalhadas são construídas como MultiMeshes, o que mantém milhares delas baratas de renderizar.

## malha de navegação { #navigation-mesh data-forms="malhas de navegação" }
Um contorno simplificado dos pisos em que agentes de IA podem andar, que o Godot usa para achar caminhos. O TrenchDotNav calcula uma a partir da colisão de um mapa construído.

## mapa normal { #normal-map data-forms="mapas normais" }
Uma textura que guarda para onde cada pixel de uma superfície aponta, para que faces planas peguem a luz como se tivessem relevos e sulcos. Ela adiciona detalhe fino sem geometria extra.

## operando { #operand data-forms="operandos" }
O número com que um cálculo trabalha além daquele que ele recebe. Em um math_calc que soma 5, o operando é 5.

## ORM { #orm }
Uma textura que empacota três mapas em tons de cinza nos canais de cor dela: oclusão de ambiente no vermelho, rugosidade no verde e metálico no azul. Materiais do Godot e camadas de terreno do TrenchDot a leem como um só arquivo.

## entidade ponto { #point-entity data-forms="entidades ponto" }
Uma entidade que fica em um único lugar em vez de tirar sua forma de brushes, como uma luz, um som ou um ponto de spawn. Posicione uma arrastando-a do painel Entidades para uma vista.

## prefab { #prefab data-forms="prefabs" }
Uma peça reutilizável de um mapa, salva como seu próprio arquivo .tdm e posicionada em outros mapas como instâncias. Editar o arquivo do prefab atualiza todas as instâncias dele.

## res:// { #res }
A forma do Godot de escrever um caminho dentro da pasta do projeto, seja qual for o nome da pasta no disco. res://maps/room.tdm é o room.tdm na pasta maps do seu projeto Godot. É por isso que mapas e modelos precisam ser salvos dentro do projeto para o addon os encontrar.

## modo restrito { #restricted-mode }
Como o TrenchDot abre um projeto Godot que contém código e em que você ainda não confiou. Você pode ver e editar os mapas dele, mas o editor não inicia o Godot nele, não constrói no Godot nem envia edições ao vivo, já que o Godot executa os scripts de um projeto ao abri-lo, executá-lo ou construí-lo. Clique em Modo restrito na barra de status para confiar no projeto.

## conjunto de scatter { #scatter-set data-forms="conjuntos de scatter" }
Um grupo de modelos pintados juntos com a ferramenta Scatter, como uma floresta ou um campo de grama, com camada, densidade e superfícies de destino próprias. O Godot constrói cada conjunto como MultiMeshes.

## reflexos no espaço da tela { #screen-space-reflections }
Um efeito do Godot que espelha em superfícies brilhantes o que já está na tela, para que ruas molhadas reflitam as lâmpadas. Ligue com a key ssr do worldspawn definida como 1.

## sinal { #signal data-forms="sinais" }
O nome que o Godot dá a um evento que um nó anuncia, como um botão sendo pressionado. Todo output de entidade é um sinal, e ligar um output conecta esse sinal a um método de input no alvo.

## targetname { #targetname }
O nome que você dá a uma entidade para que outras entidades possam alcançá-la, como main_door. Outputs, fixups de prefab e keys como fixture encontram entidades por ele.

## texel { #texel data-forms="texels" }
Um pixel de uma textura como ele cai em uma superfície. Por padrão, um texel cobre uma unidade de mapa, e a densidade de texels, quantos texels cabem em uma área, decide a nitidez de uma textura. Um lightmap também tem texels, e o tamanho do texel diz quantas unidades de mapa cada um cobre.

## tamanho de textura { #texture-size data-forms="tamanhos de textura" }
O tamanho no mundo que uma repetição de textura cobre, definido como metadata/texture_size em um material do Godot. Sem ele, um pixel cobre uma unidade de mapa, o que estica uma foto de alta resolução sobre uma área enorme.

## texturas de ferramenta { #tool-textures data-forms="textura de ferramenta" }
Texturas especiais que dizem à construção para que serve uma face de brush em vez de desenhá-la. Faces clip só colidem, faces skip não constroem nada e faces origin definem onde fica a origem de uma entidade brush. Toda textura nas pastas special/, tools/ ou td/ não desenha nada no Godot, e o mesmo vale para nodraw, hint, trigger e os outros nomes de ferramenta em qualquer pasta.

## TrenchBroom { #trenchbroom }
Um editor de níveis gratuito para jogos no estilo Quake. O TrenchDot tira muita inspiração dele e consegue importar seus arquivos .map.

## volume de trigger { #trigger-volume data-forms="volumes de trigger" }
Uma entidade brush invisível que dispara outputs quando um corpo entra ou sai dela, por exemplo para iniciar uma cutscene quando o jogador entra em uma sala. Desenhe uma com a ferramenta Volume (Shift+E).

## trim sheet { #trim-sheet data-forms="trim sheets" }
Uma textura que empacota faixas de detalhe, como bordas, canos e painéis, em uma só imagem. As faces são encaixadas nas partes dela com hotspots, então algumas trim sheets podem texturizar um prédio inteiro.

## triplanar { #triplanar }
Uma forma de projetar uma textura em uma superfície a partir de três direções, de cima e de dois lados, misturadas pelo ângulo da superfície. As camadas de terreno a usam para que penhascos mostrem rocha em vez de um borrão esticado.

## bloqueio de UV { #uv-lock }
Um ajuste, ligado por padrão e alternado com Ctrl+Shift+U, que mantém as texturas coladas a um brush enquanto você o move. Com ele desligado, a textura fica parada no mundo e desliza pelas faces movidas.

## UV { #uv }
As coordenadas que dizem qual parte de uma textura cai em cada ponto de uma face, U ao longo da textura e V para baixo. As faces de brush normalmente as recebem de uma projeção, enquanto malhas e algumas faces guardam as suas para cada canto.

## worldspawn { #worldspawn }
A entidade raiz do mapa, que guarda os brushes do mundo. As keys dela guardam configurações do mapa todo, como céu, névoa, sol e streaming, e o Inspetor a mostra quando nada está selecionado.

## yaw { #yaw }
Rotação em torno do eixo vertical, a direção para onde algo olha visto de cima. O yaw de um destino de teleporte define para onde os corpos olham depois de chegar.

## z-fighting { #z-fighting }
Listras tremulantes onde duas superfícies estão no mesmo plano ou muito perto dele. A GPU não consegue dizer qual está na frente, então cada pixel mostra uma ou outra, e o padrão muda conforme a câmera se move. Decals evitam isso, brushes finos colocados sobre uma parede não.
