---
source: 1cc5b4057e3a84e4d2bcf666b68e70eb94fb265b
---

# Editor de UV { #uv-editor }

O Editor de UV desenha as faces selecionadas de brushes e malhas planas sobre a textura repetida delas, para você ver que parte da textura cada face mostra. Use-o para o que a [ferramenta Textura](texturing.md#the-texture-tool) não faz na vista 3D, como posicionar as UVs de uma malha com precisão de pixel.

## Abrindo { #opening-it }

O Editor de UV é uma aba no painel da direita, ao lado do Inspetor, e *Textura > Editor de UV* também o traz à frente. Depois selecione faces: Shift+clique em uma numa vista, ou use a ferramenta Textura. A linha de cima diz o nome do material e quantas faces você tem.

![O Editor de UV com uma face de parede selecionada, desenhada como um retângulo laranja sobre a textura showcase/plaster repetida](../assets/uv-editor/uv-editor-face.png)

> [!TIP]
> Enquanto o Editor de UV é a aba visível do painel, Ctrl+clique em um brush ou malha seleciona todas as faces dele, para texturizar todos os lados de uma vez. Brushes e malhas que já estavam selecionados também viram suas faces. Quando a seleção tem qualquer outra coisa, como um terreno ou uma entidade, Ctrl+clique adiciona e remove objetos como de costume.

Uma face de brush normalmente recebe a textura de um único deslocamento, escala e rotação. Uma face com **UVs explícitas** guarda uma posição de textura para cada canto, que é o que as malhas precisam. O Editor de UV desenha essas faces em azul, e você pode escolher os cantos um por um. As faces laranja não têm UVs explícitas e se movem como um todo.

## Controles { #controls }

O **?** no fim da linha de cima mostra isto também no editor.

| Entrada | Resultado |
| --- | --- |
| Arrastar o espaço vazio | Move a textura, ou as faces inteiras quando elas têm UVs explícitas |
| Roda, arrastar com o botão direito | Escala, gira |
| Clicar num canto | Seleciona-o junto com os cantos das faces vizinhas costuradas a ele |
| Shift+clique, Ctrl+clique | Adiciona, remove |
| Alt+clique | Seleciona só o canto daquela face, para desfazer uma costura |
| Shift ou Ctrl+arrastar no espaço vazio | Seleção por caixa, adicionando ou removendo |
| Arrastar um canto selecionado | Move a seleção toda |
| Clique duplo num canto | Gizmo que se move só ao longo de U ou V, Escape o esconde |
| Arrastar com o botão do meio, Ctrl+roda | Desloca, dá zoom na tela |

Com cantos selecionados, a roda e o arrastar com o botão direito os escalam e giram em torno do centro deles. Os campos U e V embaixo da tela posicionam a seleção com precisão de pixel.

Os botões pequenos depois dos menus alternam o ajuste ao pixel, mostram os hotspots da textura, fazem um arrastar simples virar seleção por caixa, mostram o gizmo, alternam o bloqueio de UV e enquadram as faces de novo. Passe o mouse sobre um para ver qual é qual.

## Projeções de UV de malha { #mesh-uv-projections }

Com faces de malha selecionadas, o menu **Projeção** monta UVs explícitas para você. Ele usa a densidade de texels que uma face de brush com a mesma textura teria, então as malhas combinam com os brushes ao redor. Passe o mouse sobre uma entrada para ler uma explicação de uma linha.

![O menu Projeção do Editor de UV aberto, Cilindro Y com o mouse em cima e sua dica, Envolve este eixo do mundo pelo centro das faces, u gira em torno dele](../assets/uv-editor/projection-menu.png)

| Comando | Resultado |
| --- | --- |
| *Planar*, *Planar (X)*, *(Y)*, *(Z)* | Projeta na direção para a qual as faces mais apontam, ou ao longo de um eixo do mundo, mantendo-as numa peça só |
| *Caixa* | Projeta cada face a partir do eixo do mundo mais próximo, para formas quadradas |
| *Cilindro X*, *Y*, *Z*, *Esfera* | Envolvem as faces com a textura, para canos, colunas e cúpulas |
| *Exibir* | Projeta direto ao longo da câmera 3D |
| *Desdobrar* | Abre as faces planas aresta por aresta, para que tiras e grades continuem contínuas |
| *Redefinir para o mundo* | A projeção de uma face de brush recém-redefinida |
| *Pré-calcular planar* | Transforma a textura planar em UVs de canto editáveis, mostrado enquanto algumas faces ainda não têm nenhuma |
| *Limpar UVs* | Descarta as UVs explícitas de novo |
| *Normalizar* | Escala as UVs para caber uma vez na textura |
| *Empacotar* | Empacota as ilhas na textura com uma só densidade de texels |

Aqui uma coluna de malha de 12 lados recebeu *Cilindro Y*, e os lados dela agora ficam lado a lado como uma tira só em volta da textura:

![O Editor de UV depois de Cilindro Y numa coluna de malha de 12 lados, as faces laterais dispostas numa tira contínua](../assets/uv-editor/uv-editor-cylinder.png)

O menu **UV** inverte, gira, ajusta, alinha, endireita e ajusta ao pixel. O menu **Hotspot** ajusta a um retângulo de hotspot, veja [Hotspots](texturing.md#hotspots).
