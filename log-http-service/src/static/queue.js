/**
 * API для работы с очередю
 */
class QueueApi {
    constructor(base) {
        this.base = base
    }
    
    /** 
     * @returns promise with { debug: string, crate_name: string, crate_ver: string }
     */
    version() {
        return fetch(this.base + '/version').then(res => {
            if( res.ok ){
                return res.json()
            }else{
                return Promise.reject("not succ executred")
            }
        })
    }

    /** @return promise with { log_id: string, block_id: string }  */
    currentId() {
        return fetch(this.base + '/tail/id').then(res => {
            if( res.ok ){
                return res.json()
            }else{
                return Promise.reject("not succ executred")
            }
        })
    }

    /** 
     * fetch last N records 
     * 
     * @returns promise with 
     * @example
     * 
     *     {
     *       "values": [
     *         {
     *           "rid": {
     *             "log_id": "0",
     *             "block_id": "3"
     *           },
     *           "result": {
     *             "Succ": {
     *               "log_file": "/home/user/code/rust/raft/log-http-service/app_data/queue/2023-07-21T18-29-39-a9p29.binlog",
     *               "log_id": "0",
     *               "block_id": "3",
     *               "options": {
     *                 "mime": "text/plain",
     *                 "encoding": "utf-8",
     *                 "time": "2023-07-21T22:37:38.481669+00:00"
     *               },
     *               "position": "380",
     *               "head_size": 135,
     *               "data_size": 7,
     *               "tail_size": 8,
     *               "preview": "bla bla"
     *             }
     *           }
     *         },
     *         {
     *           "rid": {
     *             "log_id": "0",
     *             "block_id": "2"
     *           },
     *           "result": {
     *             "Succ": {
     *               "log_file": "/home/user/code/rust/raft/log-http-service/app_data/queue/2023-07-21T18-29-39-a9p29.binlog",
     *               "log_id": "0",
     *               "block_id": "2",
     *               "options": {
     *                 "time": "2023-07-21T22:29:36.327761+00:00",
     *                 "encoding": "utf-8",
     *                 "mime": "text/plain"
     *               },
     *               "position": "231",
     *               "head_size": 135,
     *               "data_size": 6,
     *               "tail_size": 8,
     *               "preview": "hello2"
     *             }
     *           }
     *         }
     *       ]
     *     }
     */
    lastHeadersN(count) {
        return fetch(this.base + '/headers/last/'+count).then(res => {
            if( res.ok ){
                return res.json()
            }else{
                return Promise.reject("not succ executred")
            }
        })
    }

    /** 
     * insert plain text at end of log 
     * @returns promise with { log_id: string, block_id: string }
    */
    insertPlain(text) {
        return fetch(this.base + '/insert/plain', {
            method: 'POST',
            headers: {
                'Content-Type': 'text/plain'
            },
            cache: 'no-cache',
            body: text
        }).then(res => {
            if( res.ok ){
                return res.json()
            }else{
                return Promise.reject("not succ executred")
            }
        })
    }

    /** 
     * fetch log files info 
     * @returns promise with
     * 
     * @example
     * 
     *     {
     *       "files": [
     *         {
     *           "log_id": "0",
     *           "log_file": "/2023-07-21T18-29-39-a9p29.binlog",
     *           "items_count": 8,
     *           "bytes_count": 1138
     *         },
     *         {
     *           "log_id": "1",
     *           "log_file": "/2023-07-26T03-06-37-qadnb.binlog",
     *           "items_count": 1,
     *           "bytes_count": 118
     *         }
     *       ]
     *     }
     */
    files() {
        return fetch(this.base + '/log/files').then(res => {
            if( res.ok ){
                return res.json()
            }else{
                return Promise.reject("not succ executred")
            }
        })
    }

    /**
     * switch log
     * @returns 
     * {
     *   "log_file": "/app_data/queue/2023-07-26T03-06-37-qadnb.binlog",
     *   "log_id": "1"
     * }
     */
    switchTail() {
        return fetch(this.base + '/tail/switch', {
            method: 'POST',
            cache: 'no-cache',
        }).then(res => {
            if( res.ok ){
                return res.json()
            }else{
                return Promise.reject("not succ executred")
            }
        })
    }

    /** 
     * Возвращает promise с blob записи
     * 
     * @argument logId - ид лога
     * @argument blockId - ид блока
     */
    rawRecord(logId, blockId) {
        return fetch(
            this.base + '/record/'+logId+'/'+blockId+'/raw' 
        ).then( res => {
            if( res.ok ){
                return res.blob()
            }else{
                return Promise.reject("not succ executed")
            }
        })
    }

    /** 
     * Добавляет blob содержащий блок записи, в конец очереди
     * 
     * Конец определяется по аргументам logId, blockId
     * 
     * @argument logId - ид лога
     * @argument blockId - ид блока
     * 
     * @returns promise
     */
    insertRaw(logId, blockId, blob) {
        return fetch(
            this.base + '/record/'+logId+'/'+blockId+'/raw', {
            method: 'POST',
            cache: 'no-cache',
            body: blob
        }).then( res => {
            if( res.ok ){
                return res.json()
            }else{
                return Promise.reject("not succ executed")
            }
        })
    }
}

var queueApi = new QueueApi('/queue')

/**
 * Формирование плана наката лога
 * 
 * @param {*} masterQueue главная очередь, источник данных
 * @param {*} slaveQueue целевая очередь, назначение
 * 
 * @returns promise 
 * 
 * @example
 * [ {action:'switch'}
 * , {action:'push', log_id:0, block_id:1 }
 * ]
 */
function rollUpLogsPlan( masterQueue, slaveQueue ) {
    function doSwitch(self) {
        let slaveQueue = this.slave        
        
        return slaveQueue.switchTail().then(()=>{
            return Promise.resolve('switched')
        }).catch(e => {
            return Promise.reject(e)
        })
    }

    function doLogPush(self) {
        let masterQueue = this.master
        let slaveQueue = this.slave
        let target = { log_id: this.log_id, block_id: this.block_id - BigInt(1) }
        let source = { log_id: this.log_id, block_id: this.block_id }
        return masterQueue.rawRecord( source.log_id, source.block_id ).then( blob => {
            self.log.push('blob fetched')
            return slaveQueue.insertRaw(target.log_id, target.block_id, blob).then(r => {
                return Promise.resolve('blob inserted')
            })
        })
    }

    function cmpId( id_a, id_b ){
        let lid_a = BigInt(id_a.log_id)
        let lid_b = BigInt(id_b.log_id)

        let bid_a = BigInt(id_a.block_id)
        let bid_b = BigInt(id_b.block_id)

        if( lid_a == lid_b ){
            if( bid_a==bid_b ){
                return 0
            }else{
                return bid_a < bid_b ? -1 : 1
            }
        }else{
            return lid_a < lid_b ? -1 : 1
        }
    }

    // Генерация последовательности идентификаторов
    function generateSeqOfRec( logId, fromBlockId, toBlockId ) {
        let seq = []
        for( let i=fromBlockId; i<=toBlockId; i++ ){
            seq.push({log_id: logId, block_id:i})
        }
        return seq
    }

    return masterQueue.currentId().then( masterId => {
        return slaveQueue.currentId().then( slaveId => {
            if( cmpId(masterId,slaveId) <= 0 ){
                // нет надобности обновлять
                return Promise.resolve([])
            } else {
                // Найти файлы (master log id = 3, slave log id = 2)
                //   log-1 - пропустить 
                //   log-2 - взять часть
                //   log-3 - взять все
                return masterQueue.files().then( masterFiles => {
                    const sourceFiles = masterFiles.files.filter( 
                        logFile => BigInt(logFile.log_id) >= BigInt(slaveId.log_id)
                    );
                    // исходные файлы должны
                    //  каждый содержать свойство items_count - число
                    //  номера log_id быть уникальны
                    //  номера log_id быть последовательны шаг +1
                    //  начинаться с slaveId.log_id

                    if( sourceFiles.filter(l => !('items_count' in l)).length ){
                        // err!
                        return Promise.reject('sourceFiles not contains items_count')
                    }

                    const srcRecIdArrArr = sourceFiles.map( logFile => {
                        let toBlockId = BigInt(logFile.items_count)
                        if( toBlockId == 1 )return []

                        toBlockId--;
                        let fromBlockId = BigInt(1)

                        if( slaveId.log_id == logFile.log_id ){
                            fromBlockId = BigInt(slaveId.block_id)+BigInt(1)
                        }

                        return generateSeqOfRec(BigInt(logFile.log_id), fromBlockId, toBlockId)
                    })

                    let plan = [] 
                    for( let i=0; i<srcRecIdArrArr.length; i++ ){
                        let step = {
                            state: 'init', 
                            log:ref([]),
                            master: masterQueue,
                            slave: slaveQueue,
                        }

                        if( i>0 ){
                            step.action = 'switch'
                            step.execute = doSwitch.bind(step)
                            plan.push(step)
                        }

                        for( let j=0; j<srcRecIdArrArr[i].length; j++ ){
                            let id = srcRecIdArrArr[i][j]
                            step = { 
                                log:[],
                                state: 'init',
                                master: masterQueue,
                                slave: slaveQueue,
                            }
                            step.action = 'push'
                            step.log_id = id.log_id
                            step.block_id = id.block_id
                            step.execute = doLogPush.bind(step)
                            plan.push(step)
                        }
                    }
                    
                    return Promise.resolve(plan)
                })
            }
        })
    })
}

