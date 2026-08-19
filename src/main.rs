
// Estrutura definida para o frame: [header][dados]
// header:  [TAMANHO DO ARQUIVO, EXTENSÃO ORIGINAL]
// corpo : [dados]

//dados -> logica do LZ4: sequencias de [sequence]
// cada sequence: [token][literals][offset] ou [token][literals]
// cada token tem 1 byte [nibble alto, nibble baixo]
// nibble alto: quantidade de literals 
// nibble baixo: tamanho do match - 4

// minmatch de 4 bytes/caracteres


//TO DO:
// Trocar estratégia de matching. Atual: Primeiro match. Ideal: Match mais longo.
// Aplicar correções (essencial antes de prosseguir para manipulacao de arquivos.)
// Implementar e adotar o uso de arquivos
// refatoração e reestruturação / organização do codigo

//CORREÇÕES:
// - Flush ao final do loop de compressao (feito parcial, necessario testes)
// - Extensao para match_len > 15 (extensão para nibble baixop)



const MINMATCH: usize = 4;


fn decompressor(  dados_dec: &Vec<u8>){
    let mut saida: Vec<u8> = Vec::new();
    
    let mut p:usize = 0;
    
    while p<dados_dec.len(){

        let token_pos: usize = p;
        let token = dados_dec[token_pos];

        let mut literal_count: u16 = ((token >>4) &0x0F )as u16;
        
        let match_len: usize = (token &0x0F) as usize;
        
        let mut offset: usize;
        
        let mut ext_bytes:usize = 0;
        
        if literal_count >=15{

            ext_bytes+=1;

            while ((dados_dec[p+ext_bytes]) as u16) >= 255{
               literal_count += dados_dec[p+ext_bytes] as u16; 
               ext_bytes+= 1;
            }
            
            if (dados_dec[p+ext_bytes] as u16) < 255 {
                literal_count += dados_dec[p+ext_bytes] as u16;
            }
            
        }

        if dados_dec.len() > p + ext_bytes + 2 + literal_count as usize{
            let l: u16 = dados_dec[p + ext_bytes + 1 + literal_count as usize] as u16;
            let h: u16 = dados_dec[p + ext_bytes + 2 + literal_count as usize] as u16;
            offset = (l | (h << 8)) as usize;
        }else{
            offset = 0;
        }
        

        saida.extend_from_slice(&dados_dec[p + ext_bytes + 1.. p + ext_bytes + (literal_count as usize) + 1]);
        
        // logica para encontrar slice a ser copiado:
        // p + literal = fim do slice, ou seja, basta subtrair o offset disso para saber onde inicia o match
        // a partir da posicao do match (p+literais-offset) basta somar o match_len + MINMATCH para saber o tamanho do match, temos:
        // dados[p + literal - offset .. p + literal - offset + match_len + MINMATCH]
        // 

        if offset > 0{
            let slice_dec : Vec<u8> =  saida[saida.len() - offset .. saida.len() - offset + match_len +  MINMATCH ].to_vec();
            saida.extend_from_slice(&slice_dec);
        }
        

        //println!("Saida (descomprimida): {}", String::from_utf8_lossy(&saida));
        
        //verificar p (posição no comeco da sequencia de literais)
        p = p + literal_count as usize + ext_bytes + 3;

    }

    println!("Saida (descomprimida): {}", String::from_utf8_lossy(&saida));

    
}





//              COMPRESSOR
fn main() {
    
    //let texto:String = String::from("Ola, computador, computador. Este e um texto de teste para testar o compressor manual do Clubix!. Qualquer semelhanca com outro compressor e mera coincidencia. computadores sao legais! Um compressor serve, principalmente, para comprimir arquivos. A ideia e que eles gastem a menor quantidade de espaco possivel no disco do computador.Ola Mundo.");
    //let dados: &[u8] = texto.as_bytes();
    
    let arquivo =std::fs::read("shrek2.txt"); 
    let dados = match arquivo{
        Ok(T) => T,
        Err(err) =>{
            println!("Erro");
            return ();
        }
    };


    let mut saida:Vec<u8> = Vec::new();

    let mut p:usize = 0;
    let mut p_end:usize = MINMATCH;

    let mut literal_count:u16 = 0;
    

    let mut token_pos: usize = 0;
    
    
    while p_end<dados.len(){
        
        //println!(" Byte atual[{}..{}] -> {} ", p, p_end, String::from_utf8_lossy(&dados[p..p_end]));

        // apontam para intervalo onde estamos buscando o match atual
        let mut count:usize = p.saturating_sub(65535);
        let mut count_end:usize = count + MINMATCH;
        
        //apontam para intervalo onde está o maior match encontrado
        let mut m:usize = count;
        let mut m_end: usize = count_end;
        let mut m_size:usize = 0;


        //confirma se existe match para o loop/busca atual 
        let mut b_match:bool = false;
        

        while count_end<p{
            while true {
                if p_end-p<=15 && dados[count..count_end] == dados[p..p_end] && count_end<=p{

                    //println!("Match encontrado -> [{}..{}] == [{}..{}] | {}", count, count_end, p, p_end, String::from_utf8_lossy(&dados[count..count_end]));
                    if (count_end - count) > m_size{
                        m = count;
                        m_end = count_end;
                        m_size = m_end - m;
                    }
                    b_match = true;
                    
                }else{
                    p_end=p+MINMATCH;
                    break;
                }
                count_end += 1;
                p_end += 1;               
            }


            count +=1 ;
            count_end = count + MINMATCH;

        }

        if b_match==true {
            
            // transferir count de literais para nible alto
            // criar nible baixo com o offset / match len
            //adicionar / alterar token
            
            //println!("{}",literal_count);

            let match_len:u8 = (m_size - MINMATCH) as u8;
            
            let offset: u16= (p-m) as u16;

            let mut token:u8; 

            if literal_count>=15{

                token = ((15& 0x0F) << 4) | (match_len& 0x0F);

                saida.push(token);

                literal_count-=15; 
                
                while literal_count>=255{
                    saida.push(255);
                    literal_count= literal_count.saturating_sub(255);
                }
                saida.push(literal_count as u8);

            }else{

                token = (((literal_count) as u8 & 0x0F) << 4) | (match_len& 0x0F);
                saida.push(token);

            }


            saida.extend_from_slice(&dados[token_pos..p]);
            saida.extend_from_slice(&offset.to_le_bytes());


            literal_count = 0;
            p +=  (match_len) as usize + (MINMATCH) as usize;
            p_end = p + MINMATCH;
            token_pos = p;

            //println!("token -> literals:{} | Tamanho do match:{} | Offset: {}",(token >> 4) & 0x0F, token & 0x0F, offset);

        }else{
            p = p + 1;
            p_end = p + MINMATCH;
            literal_count+=1;
        }
        
    }
    
    // flush final apos loop para garantir saida de todos os bytes do arquivo
    // criar token com nibble baixo = 0
    // 
    let mut token:u8;

    if literal_count>0{
        println!("Faltou flush...");
        println!("Flushing...");
        println!("Posição do ponteiro p {}", token_pos);
        
        literal_count = (dados[token_pos ..].len()) as u16;

        if literal_count  > 15{
            println!("Explodiu");
            token = ((15& 0x0F) << 4) | (0 & 0x0F);
            saida.push(token);
            literal_count-=15;

            while literal_count>=255{
                saida.push(255);
                literal_count= literal_count.saturating_sub(255);
            }
            saida.push(literal_count as u8);
            
            saida.extend_from_slice(&dados[token_pos .. ]);


        }else{
            token = ((literal_count as u8 & 0x0F) << 4) | (0 & 0x0F);
            saida.push(token);
            saida.extend_from_slice(&dados[token_pos .. ]);
        }

    }
    
    println!("Tamanho final da saída {} | entrada {}  | Taxa de compressao: {} ", saida.len(), dados.len(), (1 as f32 - (saida.len()) as f32/(dados.len()) as f32) );
    //println!("Saida (bytes): {:?}", saida);
    //println!("Saida (lossy string): {}", String::from_utf8_lossy(&saida));

    decompressor(&saida);

}
