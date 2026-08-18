
// Estrutura definida para o frame: [header][dados]
// header:  [TAMANHO DO ARQUIVO, EXTENSÃO ORIGINAL]
// corpo : [dados]

//dados -> logica do LZ4: sequencias de [sequence]
// cada sequence: [token][literals][offset] ou [token][literals]
// cada token tem 1 byte [nibble alto, nibble baixo]
// nibble alto: quantidade de literals 
// nibble baixo: tamanho do match - 4

// minmatch de 4 bytes/caracteres


const MINMATCH: usize = 4;


fn decompressor(  dados_dec: &Vec<u8>){
    let mut saida: Vec<u8> = Vec::new();
    
    let mut p:usize = 0;
    
    while true{
        let token_pos: usize = p;
        let token = dados_dec[token_pos];

        let mut aux: usize = p ;

        let mut literal_count: u16 = ((token >>4) &0x0F )as u16;
        let match_len: usize = (token &0x0F) as usize;
        let mut offset: usize;
        
        if literal_count >=15{
            aux+=1;

            while ((dados_dec[aux]) as u16) >= 255{
               literal_count+=dados_dec[aux] as u16; 
               aux+=1;
            }
            
            if (dados_dec[aux] as u16) < 255 {
                literal_count += dados_dec[aux] as u16;
            }
            
            p = aux;
        }

        println!("Token {} ... Literais encontrados: {}", p, literal_count);
        

    }
    
}

fn main() {
    
    let texto:String = String::from("Ola, computador, computador. Este e um texto de teste para testar o compressor manual do Clubix!. Qualquer semelhanca com outro compressor e mera coincidencia. computadores sao legais!");
    let dados: &[u8] = texto.as_bytes();

    let mut saida:Vec<u8> = Vec::new();

    let mut p:usize = 0;
    let mut p_end:usize = MINMATCH;

    let mut literal_count:u16 = 0;
    

    let mut token_pos: usize = 0;
    
    while p_end<dados.len(){
        
        println!(" Byte atual[{}..{}] -> {} ", p, p_end, std::str::from_utf8(&dados[p..p_end]).unwrap());

        let mut count:usize = p.saturating_sub(65535);
        let mut count_end:usize = count + MINMATCH;
        
        let mut b_match:bool = false;
        

        while count_end<p{
            while true {
                if dados[count..count_end] == dados[p..p_end] && count_end<=p{

                    println!("Match encontrado -> [{}..{}] == [{}..{}] | {}", count, count_end, p, p_end, std::str::from_utf8(&dados[count..count_end]).unwrap());
                    b_match = true;
                    
                }else{
                    count_end-=1;
                    p_end=p+MINMATCH;
                    break;
                }
                count_end += 1;
                p_end += 1;               
            }


            if b_match==true{
                break;
            }

            count = count_end ;
            count_end = count + MINMATCH;

        }

        if b_match==true {
            
            // transferir count de literais para nible alto
            // criar nible baixo com o offset / match len
            //adicionar / alterar token
            
            println!("{}",literal_count);

            let match_len:u8 = (count_end - count - MINMATCH) as u8;
            
            let offset: u16= (p-count) as u16;

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

            println!("token -> literals:{} | Tamanho do match:{} | Offset: {}",(token >> 4) & 0x0F, token & 0x0F, offset);

        }else{
            p = p + 1;
            p_end = p + MINMATCH;
            literal_count+=1;
        }
        
    }

    decompressor(&saida); 

}
