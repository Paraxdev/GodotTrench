---
source: b3ed6fe92fc841e610e206dd64cc4020bf717ee7
---

# Arquivos danificados { #damaged-files }

Um mapa binário danificado só perde os chunks que foram danificados, todo o resto ainda abre. Se você escreve um leitor, esta página diz o que ele precisa fazer para se comportar do mesmo jeito.

Um leitor percorre os [chunks](container.md#chunks) a partir do byte 12. Um cabeçalho de chunk válido tem o marcador de sincronia, uma verificação de cabeçalho que confere, um tamanho bruto de no máximo 1 GiB e uma tag imprimível. Onde não há cabeçalho válido, o leitor procura adiante o próximo marcador de sincronia e continua a partir dele.

Um chunk `HEAD`, `NODE` ou `END` é descartado quando o payload dele não descomprime, falha no checksum, tem o tamanho errado ou não decodifica.

| Dano | Resultado |
| --- | --- |
| Um chunk `NODE` | Os nós guardados nele se perdem |
| O pai de um nó se perdeu | O nó vai, com tudo o que está abaixo dele, para uma layer chamada `Recovered` no fim do mapa. A layer ganha um id novo ao carregar |
| Aninhamento com mais de 60 níveis | Também vai para `Recovered`, para que a leitura não estoure a pilha. Só um arquivo forjado tem isso |
| O chunk `HEAD` | As propriedades do worldspawn e o estado do editor se perdem, e a versão do mapa vem do `END` |
| `HEAD` e `END` juntos | O arquivo é recusado |
| Sem chunk `END` | O arquivo foi cortado. Tudo até o último chunk completo carrega |
| Contagens do `END` acima do que foi lido | Relatado como o número de nós, ou de chunks, que faltam |

Cada leitor relata todas as perdas:

| Leitor | Relatório |
| --- | --- |
| Editor | Abre o que conseguiu ler, lista as perdas na barra de status e marca o mapa como não salvo. Prefabs danificados também são relatados na barra de status |
| Addon do Godot | Constrói o que conseguiu ler e imprime um aviso por perda, com o prefixo `[TDM]` e o caminho do arquivo |
| `trenchdot --dump` | Imprime o que conseguiu ler e escreve as perdas no stderr |

Antes de cada salvamento, o editor copia o arquivo antigo para `<map>.tdm.bak`, então salvar um mapa recuperado ainda mantém o original danificado ao lado. O arquivo novo vai primeiro para `<map>.tdm.tmp` e só depois é renomeado por cima do mapa, assim uma falha no meio do salvamento nunca deixa um mapa pela metade.
