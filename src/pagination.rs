use alloc::vec::Vec;
/// Keep a whole skill together when possible; repeat its header on continuation pages.
pub fn paginate<T:Clone>(groups:&[(usize,Vec<T>)],capacity:usize,separator:&T)->Vec<Vec<T>>{
    assert!(capacity>=3);
    let mut pages=Vec::new();let mut page=Vec::new();
    for (headers,rows) in groups {
        let headers=(*headers).min(rows.len()).min(capacity-1);
        if rows.len()<=capacity {
            if !page.is_empty()&&page.len()+1+rows.len()>capacity{pages.push(page);page=Vec::new();}
            if !page.is_empty(){page.push(separator.clone());}
            page.extend_from_slice(rows);continue
        }
        if !page.is_empty(){pages.push(page);page=Vec::new();}
        page.extend_from_slice(&rows[..headers]);
        for row in &rows[headers..]{
            if page.len()==capacity{pages.push(page);page=Vec::new();page.extend_from_slice(&rows[..headers]);}
            page.push(row.clone());
        }
    }
    if !page.is_empty(){pages.push(page)}
    pages
}
