<details>
<summary>XSD contract: <code>DimensionListType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="DimensionListType">
		<xs:annotation>
			<xs:documentation>DimensionListType describes the key descriptor for a data structure definition. The order of the declaration of child dimensions is significant: it is used to describe the order in which they will appear in data formats for which key values are supplied in an ordered fashion (exclusive of the time dimension, which is not represented as a member of the ordered key). Any data structure definition which uses the time dimension should also declare a frequency dimension, conventionally the first dimension in the key (the set of ordered non-time dimensions). If is not necessary to assign a time dimension, as data can be organised in any fashion required.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:extension base="DimensionListBaseType">
				<xs:sequence>
					<xs:element ref="Dimension" maxOccurs="unbounded"/>
					<xs:element ref="TimeDimension" minOccurs="0"/>
				</xs:sequence>
			</xs:extension>
		</xs:complexContent>
	</xs:complexType>
```

</details>
