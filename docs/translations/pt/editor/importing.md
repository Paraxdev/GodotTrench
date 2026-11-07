---
source: e0934aeaf4aaffb804a36e5ff5821a38614fc413
---

# Importando mapas do Hammer e do TrenchBroom { #importing-hammer-and-trenchbroom-maps }

Tem uma fase do Hammer ou do TrenchBroom? Traga-a para cá em vez de refazê-la. *Arquivo > Importar* abre um `.vmf` do Hammer, ou um `.map` do TrenchBroom, do Quake ou do Qodot, como um mapa novo e sem salvar (em uma aba nova se o mapa aberto tiver alterações não salvas). Salve-o como `.tdm` e o Godot o constrói como qualquer outro mapa.

## Passos { #steps }

1. Abra primeiro o projeto do Godot

    Abra o projeto para o qual o mapa se destina com *Godot > Abrir projeto do Godot…*, já que as texturas convertidas vão para a pasta de texturas dele.

2. Importe o mapa

    Vá em *Arquivo > Importar* e escolha *Arquivo .vmf (Hammer)…* ou *Arquivo .map (TrenchBroom, Quake)…*, depois o arquivo.

    ![O menu Arquivo com Importar aberto e Arquivo .vmf (Hammer)… sob o mouse, e abaixo dele Converter texturas e as importações de modelos](../assets/importing/file-import-menu.png)

3. Converta as texturas

    Quando as texturas do mapa estão perto dele, o editor pergunta "Converter as texturas do mapa?" sobre as que o projeto não tem. Responda que sim, veja [Convertendo texturas](importing-textures.md).

4. Confira a aba Problemas

    Abra a aba Problemas no canto superior esquerdo, ao lado de Outliner e Histórico. Cada material que o projeto ainda não tem aparece como um aviso, e suas faces são desenhadas como um xadrez.

    ![A aba Problemas com dois avisos, Material 'castle/brick_wall' is not in the project (12 faces), e a vista 3D com essas paredes e degraus desenhados como um xadrez cinza](../assets/importing/issues-missing-materials.png)

    As faces mantêm o nome do material, então adicionar a textura depois as corrige. Quando todas as texturas estiverem lá, esses avisos somem.

    ![O mesmo mapa depois de converter suas texturas: a aba Problemas diz Nenhum problema encontrado e as paredes mostram a textura de tijolo](../assets/importing/textures-converted.png)

5. Salve

    Use *Arquivo > Salvar como…* e salve como um `.tdm` dentro do projeto.

> [!NOTE]
> As unidades são mantidas como estão. Os mapas do Source são feitos para cerca de 40 unidades por metro e os do Quake para cerca de 32, então um mapa do Source sai um pouco maior do que no jogo dele, a menos que o projeto use outra escala.

## O que a importação mantém { #what-the-import-keeps }

| Hammer | No TrenchDot |
| --- | --- |
| Materiais de ferramenta (`tools/toolsnodraw`, `toolsclip`, `toolsplayerclip`, `toolstrigger`, `toolsorigin`, ...) | As texturas de skip, clip e origin do projeto e `special/trigger` |
| Faces de céu | A textura de céu do projeto, `special/sky` por padrão |
| Displacements | Displacements com suas alturas e alphas de mistura |
| `func_instance` | Um grupo com o conteúdo da instância, movido para o lugar. Os nomes lá dentro recebem o fixup da instância, as substituições com `$` e `#` são preenchidas, e as saídas voltadas para `instance:name;Input` são ligadas à entidade real |
| Visgroups | Camadas |
| Objetos ocultos | Nós ocultos |
| Chaves e saídas de entidades | Propriedades e saídas de entidades |

As camadas e os grupos do TrenchBroom continuam sendo camadas e grupos, e as texturas de ferramenta do Quake viram as texturas de ferramenta do projeto do mesmo jeito.

Precisa só de alguns brushes? Copie-os no TrenchBroom e aperte Ctrl+V no TrenchDot, eles vêm convertidos do mesmo jeito, veja [Biblioteca de prefabs](prefab-library.md#pasting-from-trenchbroom).

As faces de céu colidem no Godot, mas não desenham nada. A chave `sky_source` do worldspawn guarda a textura que elas tinham, e converter as texturas a transforma em um céu procedural, veja [Convertendo texturas](importing-textures.md#skies).

## Limitações conhecidas { #known-limitations }

| Limitação | O que fazer |
| --- | --- |
| Texturas dentro de arquivos `.vpk` ou `.pak` não são lidas | Extraia-as antes, por exemplo com o VPKEdit |
| As instâncias são procuradas ao lado do mapa e nas pastas acima dele | Mantenha a estrutura da pasta `maps` do jogo. Instâncias que não são encontradas continuam como entidades `func_instance` |
| As classes de entidade da Valve e do Quake não têm definições em um projeto novo, então o editor não conhece as chaves delas | Adicione-as ao seu FGD, ou troque-as por classes suas. A aba Problemas as lista |
| Alguns recursos de material do Source ficam de fora: misturas de `$basetexture2`, texturas de detalhe, env maps e proxies | Configure-os no Godot |
| Os patches do Quake 3 (superfícies curvas) e as primitivas de brush são ignorados. A barra de status diz quantos | Refaça-os como malhas |
| Displacements cujos vértices se movem para os lados, e não só para cima e para baixo, viram malhas sem alphas de mistura | Edite-os como malhas |
