const { createApp, ref } = Vue

const app = createApp({
    setup() {
      return {
        count: ref(0),
        version: ref({
            debug: false,
            name: 'no-name',
            ver: 'no-ver',
        }),
        currentId: ref({
            logId: 'no-log-id',
            blockId: 'no-block-id'
        }),
        lastHeadersNcount: ref(10),
        lastHeadersN: ref([]),
        insertPlainText: ref(''),
        insertPlainId: ref({
            logId: '',
            blockId: '',
        }),
      }
    },
    methods: {
        getVersion() {
            queueApi.version().then( d => {
                this.version.debug = d.debug
                this.version.name = d.crate_name
                this.version.ver = d.crate_ver
            });
        },
        getCurrentId() {
            queueApi.currentId().then( d => {
                this.currentId.logId = d.log_id
                this.currentId.blockId = d.block_id
            })
        },
        getLastN() {
            let cnt = this.lastHeadersNcount
            if( typeof(cnt)!="number" ){ 
                cnt = 10 
                this.lastHeadersNcount = cnt
            }
            if( cnt<1 ) {
                cnt = 1
                this.lastHeadersNcount = cnt
            }
            queueApi.lastHeadersN(cnt).then( data => {
                this.lastHeadersN.splice(0)
                data.values.forEach(element => {
                    this.lastHeadersN.push(element)
                });
            })
        },
        postPlainText() {
            queueApi.insertPlain(this.insertPlainText).then(d => {
                this.insertPlainId.logId = d.log_id
                this.insertPlainId.blockId = d.block_id
                this.getLastN()
            })
        }
    }    
  }).mount('#app')

window.addEventListener('load', (e)=>{
    app.getVersion()
    app.getCurrentId()
    app.getLastN()
});
